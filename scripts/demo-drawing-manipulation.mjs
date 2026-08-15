/**
 * Record screenshots + a GIF of Drawing UI and MCP drawingCommand manipulation.
 * Browser path only (no native OCCT). Writes under /opt/cursor/artifacts.
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
  await page.screenshot({
    path: path.join(ARTIFACTS, name),
    fullPage: true,
  });
  console.log('[demo] screenshot', name);
}

async function hold(page, ms) {
  await page.waitForTimeout(ms);
}

function convertVideoToGif(videoPath, gifPath) {
  const result = spawnSync(
    'ffmpeg',
    [
      '-y',
      '-i',
      videoPath,
      '-vf',
      'fps=8,scale=960:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=96[p];[s1][p]paletteuse=dither=bayer',
      '-loop',
      '0',
      gifPath,
    ],
    { encoding: 'utf8' },
  );
  if (result.status !== 0) {
    throw new Error(`ffmpeg gif failed: ${result.stderr?.slice(-800)}`);
  }
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
      recordVideo: {
        dir: VIDEO_DIR,
        size: { width: 1600, height: 1000 },
      },
    });
    const page = await context.newPage();
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
    await hold(page, 600);
    await shot(page, 'manipulation_01_solid_workspace.png');

    console.log('[demo] open Drawing workspace');
    await page.getByRole('button', { name: 'Drawing', exact: true }).click();
    await page.getByTestId('drawing-sheet-setup').waitFor({ timeout: 30_000 });
    await hold(page, 700);
    await shot(page, 'manipulation_02_sheet_setup.png');

    console.log('[demo] create blank sheet');
    await page.getByRole('button', { name: 'Create blank sheet' }).click();
    await page.getByTestId('drawing-workspace').waitFor({ timeout: 30_000 });
    await hold(page, 800);
    await shot(page, 'manipulation_03_blank_sheet.png');

    console.log('[demo] Auto Layout ribbon');
    await page.getByRole('button', { name: 'Auto Layout' }).click();
    await page.waitForFunction(
      () => (window.__appStore.getState().drawingDocument.sheets[0]?.views.length ?? 0) === 4,
      null,
      { timeout: 15_000 },
    );
    await hold(page, 900);
    await shot(page, 'manipulation_04_auto_layout.png');

    console.log('[demo] MCP drawingCommand add_note + update_sheet');
    await page.evaluate(async () => {
      const { getEngine } = await import('/src/engine/index.ts');
      const engine = await getEngine();
      let drawing = await engine.drawingCommand({
        op: 'add_note',
        position: [28, 22],
        text: 'MCP note',
      });
      drawing = await engine.drawingCommand({
        op: 'update_sheet',
        patch: {
          name: 'MCP Sheet',
          title_block: {
            ...drawing.sheets[0].title_block,
            title: 'MCP manipulated',
            revision: 'B',
          },
        },
      });
      await window.__appStore.getState().setDrawingDocument(drawing);
      window.__appStore.getState().setActiveTab('drawing');
    });
    await page.waitForFunction(
      () => {
        const sheet = window.__appStore.getState().drawingDocument.sheets[0];
        return sheet?.title_block?.title === 'MCP manipulated'
          && sheet.annotations.some((item) => item.kind === 'note' && item.text === 'MCP note');
      },
      null,
      { timeout: 15_000 },
    );
    await hold(page, 1000);
    await shot(page, 'manipulation_05_mcp_note_and_title.png');

    console.log('[demo] MCP delete isometric view');
    await page.evaluate(async () => {
      const { deleteDrawingView } = await import('/src/drawing/document.ts');
      const iso = window.__appStore.getState().drawingDocument.sheets[0].views
        .find((view) => view.kind === 'isometric');
      if (!iso) throw new Error('missing isometric view');
      await deleteDrawingView(iso.id);
    });
    await page.waitForFunction(
      () => !(window.__appStore.getState().drawingDocument.sheets[0]?.views ?? [])
        .some((view) => view.kind === 'isometric'),
      null,
      { timeout: 15_000 },
    );
    await hold(page, 1600);
    await shot(page, 'manipulation_06_after_delete_view.png');
    await hold(page, 800);

    const video = page.video();
    await context.close();
    const rawVideo = await video.path();
    const mp4Path = path.join(ARTIFACTS, 'drawing_mcp_manipulation.mp4');
    const gifPath = path.join(ARTIFACTS, 'drawing_mcp_manipulation.gif');
    const mp4 = spawnSync(
      'ffmpeg',
      ['-y', '-i', rawVideo, '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-an', mp4Path],
      { encoding: 'utf8' },
    );
    if (mp4.status !== 0) {
      throw new Error(`ffmpeg mp4 failed: ${mp4.stderr?.slice(-800)}`);
    }
    convertVideoToGif(rawVideo, gifPath);
    console.log('[ok] wrote', mp4Path, gifPath);
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
  console.error('[fail] drawing manipulation demo:', error);
  process.exitCode = 1;
  setTimeout(() => process.exit(1), 500);
});
