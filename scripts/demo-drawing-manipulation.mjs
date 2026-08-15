/**
 * Record a real Drawing + MCP manipulation: model a box, project it, then
 * mutate the sheet through UI ribbon + engine.drawingCommand.
 */
import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { chromium } from 'playwright';

const PORT = 7198;
const ROOT = process.cwd();
const SESSION_DIR = fs.mkdtempSync(path.join(os.tmpdir(), 'nbcad-drawing-demo-'));
const BASE = `http://127.0.0.1:${PORT}`;
const ARTIFACTS = '/opt/cursor/artifacts';
const VIDEO_DIR = fs.mkdtempSync(path.join(os.tmpdir(), 'nbcad-drawing-video-'));

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForServer(url, timeoutMs = 60_000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const response = await fetch(url);
      if (response.ok || response.status === 404) return;
    } catch {
      // retry
    }
    await sleep(250);
  }
  throw new Error(`server did not become ready: ${url}`);
}

async function shot(page, name) {
  fs.mkdirSync(ARTIFACTS, { recursive: true });
  await page.screenshot({ path: path.join(ARTIFACTS, name), fullPage: true });
  console.log('[demo] screenshot', name);
}

async function hold(page, ms) {
  await page.waitForTimeout(ms);
}

function convertVideo(rawVideo, mp4Path, gifPath) {
  const mp4 = spawnSync(
    'ffmpeg',
    ['-y', '-i', rawVideo, '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-an', mp4Path],
    { encoding: 'utf8' },
  );
  if (mp4.status !== 0) throw new Error(`ffmpeg mp4 failed: ${mp4.stderr?.slice(-800)}`);
  const gif = spawnSync(
    'ffmpeg',
    [
      '-y',
      '-i',
      rawVideo,
      '-vf',
      'fps=10,scale=1100:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer',
      '-loop',
      '0',
      gifPath,
    ],
    { encoding: 'utf8' },
  );
  if (gif.status !== 0) throw new Error(`ffmpeg gif failed: ${gif.stderr?.slice(-800)}`);
}

async function modelBracket(page) {
  const result = await page.evaluate(async () => {
    const { getEngine } = await import('/src/engine/index.ts');
    const engine = await getEngine();
    const store = window.__appStore.getState();
    store.applySolidUpdate(await engine.newProject());
    await engine.beginSketch({ type: 'origin_plane', plane: 'xy' });
    await engine.setGridSnap(false);
    await engine.addRectangle({
      mode: 'two_point',
      p1: { x: -20, y: -14 },
      p2: { x: 20, y: 14 },
      ctrl_held: true,
    });
    let ended = await engine.endSketch();
    store.setDocument(ended.document);
    store.setFinishedSketches(await engine.finishedSketches());
    let update = await engine.extrude({
      sketch_name: 'Sketch1',
      profile_indices: [0],
      operation: 'new_body',
      extent: { type: 'distance', distance: 18 },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    });
    store.applySolidUpdate(update);
    const box = update.scene.bodies[0];
    if (!box) throw new Error('extrude did not create a body');
    const vertical = box.edges.find((edge) => {
      const start = edge.points[0];
      const end = edge.points[edge.points.length - 1];
      return edge.refinable
        && Math.abs(start.x - end.x) < 1e-5
        && Math.abs(start.y - end.y) < 1e-5
        && Math.abs(start.z - end.z) > 15;
    });
    if (vertical) {
      update = await engine.solidChamfer({
        body_id: box.id,
        edge_ids: [vertical.id],
        distance: 4,
        tangent_chain: false,
      });
      store.applySolidUpdate(update);
    }
    await engine.beginSketch({ type: 'origin_plane', plane: 'xy' });
    await engine.setGridSnap(false);
    await engine.addCircle({
      mode: 'center_diameter',
      p1: { x: 36, y: 0 },
      p2: { x: 44, y: 0 },
      ctrl_held: true,
    });
    ended = await engine.endSketch();
    store.setDocument(ended.document);
    store.setFinishedSketches(await engine.finishedSketches());
    store.applySolidUpdate(await engine.extrude({
      sketch_name: 'Sketch2',
      profile_indices: [0],
      operation: 'new_body',
      extent: { type: 'distance', distance: 22 },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }));
    const scene = window.__appStore.getState().solidScene;
    return {
      bodies: scene.bodies.map((body) => ({
        id: body.id,
        name: body.name,
        edges: body.edges.length,
        faces: body.faces.length,
      })),
    };
  });
  if (!result.bodies.length) throw new Error(`model fixture failed: ${JSON.stringify(result)}`);
  return result;
}

async function addLinearDimension(page) {
  return page.evaluate(async () => {
    const { getEngine } = await import('/src/engine/index.ts');
    const engine = await getEngine();
    const state = window.__appStore.getState();
    const sheet = state.drawingDocument.sheets[0];
    const front = sheet.views.find((view) => view.kind === 'front');
    const body = state.solidScene.bodies[0];
    if (!front || !body) throw new Error('missing front view or body');
    const edge = body.edges.find((candidate) => {
      if (candidate.points.length < 2) return false;
      const start = candidate.points[0];
      const end = candidate.points[candidate.points.length - 1];
      return Math.abs(start.z - end.z) < 1e-4
        && Math.hypot(start.x - end.x, start.y - end.y) > 20;
    });
    if (!edge) throw new Error('no horizontal box edge for a linear dimension');
    const start = edge.points[0];
    const end = edge.points[edge.points.length - 1];
    const anchor = (point, endpoint) => ({
      body_id: body.id,
      edge_id: edge.id,
      edge_key: edge.key,
      endpoint,
      fallback_point: [point.x, point.y, point.z],
      circle_center: false,
    });
    const drawing = await engine.drawingCommand({
      op: 'add_linear_dimension',
      view_id: front.id,
      first: anchor(start, 'start'),
      second: anchor(end, 'end'),
    });
    await state.setDrawingDocument(drawing);
    return {
      viewId: front.id,
      annotations: window.__appStore.getState().drawingDocument.sheets[0].annotations.map((item) => item.kind),
    };
  });
}

async function main() {
  process.env.NBCAD_SESSION_DIR = SESSION_DIR;
  const viteBin = path.join(ROOT, 'node_modules', 'vite', 'bin', 'vite.js');
  const child = spawn(
    process.execPath,
    [viteBin, '--host', '127.0.0.1', '--port', String(PORT), '--strictPort'],
    {
      cwd: ROOT,
      env: { ...process.env, NBCAD_SESSION_DIR: SESSION_DIR },
      stdio: ['ignore', 'pipe', 'pipe'],
    },
  );
  const browser = await chromium.launch({ headless: true });
  try {
    await waitForServer(`${BASE}/`);
    const context = await browser.newContext({
      viewport: { width: 1600, height: 1000 },
      recordVideo: { dir: VIDEO_DIR, size: { width: 1600, height: 1000 } },
    });
    const page = await context.newPage();
    page.on('pageerror', (error) => console.log('[demo] pageerror', error.message));
    await page.goto(`${BASE}/`, { waitUntil: 'domcontentloaded' });
    await page.waitForFunction(
      () => Boolean(window.__appStore) && window.__appStore.getState().document !== null,
      null,
      { timeout: 90_000 },
    );
    await page.evaluate(async () => {
      const { getEngine } = await import('/src/engine/index.ts');
      await getEngine();
    });
    await hold(page, 500);

    console.log('[demo] model box + chamfer + boss');
    const model = await modelBracket(page);
    console.log('[demo] modeled', JSON.stringify(model));
    await page.waitForFunction(
      () => (window.__appStore.getState().solidScene.bodies.length ?? 0) >= 2,
      null,
      { timeout: 120_000 },
    );
    const iso = page.getByRole('button', { name: 'ISO', exact: true });
    if (await iso.count()) await iso.click();
    await hold(page, 1400);
    await shot(page, 'complete_01_solid_bracket.png');

    console.log('[demo] open Drawing and create A3 sheet');
    await page.getByRole('button', { name: 'Drawing', exact: true }).click();
    await page.getByTestId('drawing-sheet-setup').waitFor({ timeout: 30_000 });
    await hold(page, 400);
    const paper = page.getByLabel('Paper size');
    if (await paper.count()) await paper.selectOption('a3');
    await page.getByRole('button', { name: 'Create blank sheet' }).click();
    await page.getByTestId('drawing-workspace').waitFor({ timeout: 30_000 });
    await hold(page, 700);
    await shot(page, 'complete_02_blank_a3.png');

    console.log('[demo] Auto Layout');
    await page.getByRole('button', { name: 'Auto Layout', exact: true }).click();
    await page.waitForFunction(
      () => (window.__appStore.getState().drawingDocument.sheets[0]?.views.length ?? 0) === 4,
      null,
      { timeout: 20_000 },
    );
    await page.waitForFunction(
      () => document.querySelectorAll('[data-drawing-view-id] polyline').length >= 8,
      null,
      { timeout: 60_000 },
    );
    await hold(page, 1000);
    await shot(page, 'complete_03_projected_views.png');

    const zoom = page.locator('[data-drawing-zoom]');
    if (await zoom.count()) {
      await zoom.hover();
      await zoom.dispatchEvent('wheel', { deltaY: -80, deltaMode: 0, ctrlKey: true, clientX: 800, clientY: 500 });
      await hold(page, 500);
    }

    console.log('[demo] MCP/UI linear dimension');
    const dimensioned = await addLinearDimension(page);
    console.log('[demo] dimensioned', JSON.stringify(dimensioned));
    await page.waitForFunction(
      () => window.__appStore.getState().drawingDocument.sheets[0].annotations
        .some((item) => item.kind === 'linear_dimension'),
      null,
      { timeout: 15_000 },
    );
    await hold(page, 900);
    await shot(page, 'complete_04_linear_dimension.png');

    console.log('[demo] MCP note + title');
    await page.evaluate(async () => {
      const { addDrawingNote, updateActiveDrawingSheet } = await import('/src/drawing/document.ts');
      await addDrawingNote([36, 28], 'CHECK CHAMFER');
      await updateActiveDrawingSheet({
        name: 'Bracket Sheet',
        title_block: {
          ...window.__appStore.getState().drawingDocument.sheets[0].title_block,
          title: 'BRACKET-100',
          drawing_number: 'DWG-100',
          revision: 'B',
        },
      });
    });
    await page.waitForFunction(
      () => {
        const sheet = window.__appStore.getState().drawingDocument.sheets[0];
        return sheet.title_block.title === 'BRACKET-100'
          && sheet.annotations.some((item) => item.text === 'CHECK CHAMFER');
      },
      null,
      { timeout: 15_000 },
    );
    await hold(page, 1000);
    await shot(page, 'complete_05_note_and_title.png');

    const resetZoom = page.getByRole('button', { name: '100%', exact: true });
    if (await resetZoom.count()) await resetZoom.click();
    await hold(page, 600);

    console.log('[demo] delete isometric via drawingCommand');
    await page.evaluate(async () => {
      const { deleteDrawingView } = await import('/src/drawing/document.ts');
      const view = window.__appStore.getState().drawingDocument.sheets[0].views
        .find((item) => item.kind === 'isometric');
      if (!view) throw new Error('isometric view already missing');
      await deleteDrawingView(view.id);
    });
    await page.waitForFunction(
      () => !(window.__appStore.getState().drawingDocument.sheets[0]?.views ?? [])
        .some((view) => view.kind === 'isometric'),
      null,
      { timeout: 15_000 },
    );
    await hold(page, 1400);
    await shot(page, 'complete_06_iso_removed.png');

    const video = page.video();
    await context.close();
    const rawVideo = await video.path();
    convertVideo(
      rawVideo,
      path.join(ARTIFACTS, 'drawing_mcp_complete.mp4'),
      path.join(ARTIFACTS, 'drawing_mcp_complete.gif'),
    );
    console.log('[ok] wrote drawing_mcp_complete.mp4 and .gif');
  } finally {
    await browser.close().catch(() => undefined);
    child.kill('SIGTERM');
    try {
      fs.rmSync(SESSION_DIR, { recursive: true, force: true });
      fs.rmSync(VIDEO_DIR, { recursive: true, force: true });
    } catch {
      // best-effort
    }
    await sleep(300);
  }
}

main().catch((error) => {
  console.error('[fail] complete drawing demo:', error);
  process.exitCode = 1;
  setTimeout(() => process.exit(1), 500);
});
