/**
 * Drawing co-link smoke (browser path, no native OCCT required).
 *
 * 1. Start Vite with session HTTP bridge
 * 2. Create a sheet through the Drawing workspace UI
 * 3. Run the MCP-native `drawingCommand` path in the same WASM engine
 * 4. Publish the UI session and assert model.json contains the sheet/note
 * 5. Simulate MCP writeback of a drawing title and assert the UI applies it
 */
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { chromium } from 'playwright';

const PORT = 7197;
const ROOT = process.cwd();
const SESSION_DIR = fs.mkdtempSync(path.join(os.tmpdir(), 'nbcad-drawing-colink-'));
const BASE = `http://127.0.0.1:${PORT}`;
const ARTIFACTS = '/opt/cursor/artifacts';

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

function atomicWrite(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const temporary = `${filePath}.${process.pid}.tmp`;
  fs.writeFileSync(temporary, content, 'utf8');
  fs.renameSync(temporary, filePath);
}

async function screenshot(page, name) {
  try {
    fs.mkdirSync(ARTIFACTS, { recursive: true });
    await page.screenshot({
      path: path.join(ARTIFACTS, name),
      fullPage: true,
    });
  } catch (error) {
    console.log('[drawing-colink] screenshot skipped', name, error.message);
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
  let serverLog = '';
  const appendLog = (chunk) => {
    serverLog += chunk.toString();
    if (serverLog.length > 32_000) serverLog = serverLog.slice(-16_000);
  };
  child.stdout.on('data', appendLog);
  child.stderr.on('data', appendLog);

  const browser = await chromium.launch({ headless: true });
  try {
    await waitForServer(`${BASE}/`);
    const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });
    page.on('pageerror', (error) => {
      console.log('[drawing-colink] pageerror', error.message);
    });
    await page.goto(`${BASE}/`, { waitUntil: 'domcontentloaded' });
    await page.waitForFunction(
      () =>
        Boolean(window.__appStore) &&
        typeof window.__nbcadPublishSession === 'function' &&
        window.__appStore.getState().document !== null,
      null,
      { timeout: 90_000 },
    );

    console.log('[drawing-colink] warming engine');
    await page.evaluate(async () => {
      const { getEngine } = await import('/src/engine/index.ts');
      await getEngine();
    });

    console.log('[drawing-colink] UI create blank sheet');
    await page.getByRole('button', { name: 'Drawing', exact: true }).click();
    await page.getByTestId('drawing-sheet-setup').waitFor({ timeout: 30_000 });
    await page.getByRole('button', { name: 'Create blank sheet' }).click();
    await page.getByTestId('drawing-workspace').waitFor({ timeout: 30_000 });
    const afterUiSheet = await page.evaluate(() => {
      const drawing = window.__appStore.getState().drawingDocument;
      return {
        sheets: drawing.sheets.length,
        format: drawing.sheets[0]?.format ?? null,
        annotations: drawing.sheets[0]?.annotations.length ?? 0,
        activeTab: window.__appStore.getState().activeTab,
      };
    });
    if (afterUiSheet.sheets !== 1 || afterUiSheet.activeTab !== 'drawing') {
      throw new Error(`UI sheet setup failed: ${JSON.stringify(afterUiSheet)}`);
    }
    await screenshot(page, 'drawing_workspace_after_ui_sheet.png');

    console.log('[drawing-colink] engine.drawingCommand add_note');
    const afterNote = await page.evaluate(async () => {
      const { getEngine } = await import('/src/engine/index.ts');
      const engine = await getEngine();
      const drawing = await engine.drawingCommand({
        op: 'add_note',
        position: [24, 18],
        text: 'MCP note',
      });
      await window.__appStore.getState().setDrawingDocument(drawing);
      window.__appStore.getState().setActiveTab('drawing');
      const next = window.__appStore.getState().drawingDocument;
      const note = next.sheets[0]?.annotations.find((item) => item.kind === 'note');
      return {
        sheets: next.sheets.length,
        notes: next.sheets[0]?.annotations.filter((item) => item.kind === 'note').length ?? 0,
        text: note?.text ?? null,
      };
    });
    if (afterNote.notes < 1 || afterNote.text !== 'MCP note') {
      throw new Error(`drawingCommand note failed: ${JSON.stringify(afterNote)}`);
    }
    await page.getByTestId('drawing-workspace').waitFor();
    await screenshot(page, 'drawing_workspace_after_mcp_note.png');

    console.log('[drawing-colink] publishing UI session');
    let sessionId = null;
    const publishDeadline = Date.now() + 120_000;
    while (!sessionId && Date.now() < publishDeadline) {
      sessionId = await page.evaluate(async () => window.__nbcadPublishSession());
      if (!sessionId) await sleep(1000);
    }
    if (!sessionId) {
      throw new Error('UI did not publish a session id');
    }

    const modelPath = path.join(SESSION_DIR, sessionId, 'model.json');
    const heartbeatPath = path.join(SESSION_DIR, sessionId, 'heartbeat.json');
    const writerPath = path.join(SESSION_DIR, sessionId, 'writer.json');
    const deadline = Date.now() + 30_000;
    while (!fs.existsSync(modelPath) && Date.now() < deadline) {
      await sleep(200);
    }
    if (!fs.existsSync(modelPath)) {
      throw new Error(`model.json missing under ${SESSION_DIR}/${sessionId}`);
    }

    const published = JSON.parse(fs.readFileSync(modelPath, 'utf8'));
    const publishedSheets = published.drawings?.sheets;
    if (!Array.isArray(publishedSheets) || publishedSheets.length < 1) {
      throw new Error(
        `published model missing drawings.sheets: keys=${Object.keys(published).join(',')}`,
      );
    }
    const publishedNotes = (publishedSheets[0].annotations ?? []).filter(
      (item) => item.kind === 'note',
    );
    if (publishedNotes.length < 1 || publishedNotes[0].text !== 'MCP note') {
      throw new Error(`published model missing MCP note: ${JSON.stringify(publishedSheets[0])}`);
    }

    const heartbeat = JSON.parse(fs.readFileSync(heartbeatPath, 'utf8'));
    const nextGeneration = Number(heartbeat.generation ?? 1) + 1;
    published.drawings.sheets[0].title_block = {
      ...(published.drawings.sheets[0].title_block ?? {}),
      title: `colink-drawing-${nextGeneration}`,
    };

    atomicWrite(
      writerPath,
      JSON.stringify({
        writer: 'mcp',
        updated_ms: Date.now(),
        generation: nextGeneration,
      }),
    );
    atomicWrite(modelPath, JSON.stringify(published));
    atomicWrite(
      heartbeatPath,
      JSON.stringify({
        updated_ms: Date.now(),
        generation: nextGeneration,
        session_id: sessionId,
        session_mode: 'live',
        source: 'mcp',
      }),
    );

    console.log('[drawing-colink] waiting for UI apply', nextGeneration);
    await page.waitForFunction(
      (expected) => window.__nbcadLastMcpGeneration === expected,
      nextGeneration,
      { timeout: 120_000 },
    );
    const applied = await page.evaluate(() => {
      const drawing = window.__appStore.getState().drawingDocument;
      return {
        title: drawing.sheets[0]?.title_block?.title ?? null,
        notes: drawing.sheets[0]?.annotations.filter((item) => item.kind === 'note').length ?? 0,
        activeTab: window.__appStore.getState().activeTab,
      };
    });
    if (applied.title !== `colink-drawing-${nextGeneration}`) {
      throw new Error(`expected writeback title colink-drawing-${nextGeneration}, got ${JSON.stringify(applied)}`);
    }
    await page.evaluate(() => {
      window.__appStore.getState().setDrawingSheetSetupOpen(false);
      window.__appStore.getState().setActiveTab('drawing');
    });
    await page.getByTestId('drawing-workspace').waitFor({ timeout: 30_000 });
    await screenshot(page, 'drawing_workspace_after_mcp_writeback.png');

    console.log(
      `[ok] drawing co-link smoke passed (session=${sessionId}, generation=${nextGeneration}, notes=${applied.notes})`,
    );
  } finally {
    await browser.close().catch(() => undefined);
    child.kill('SIGTERM');
    try {
      fs.rmSync(SESSION_DIR, { recursive: true, force: true });
    } catch {
      // best-effort
    }
    await sleep(300);
    if (serverLog && process.env.COLINK_DEBUG) {
      console.log(serverLog.slice(-4000));
    }
  }
}

main().catch((error) => {
  console.error('[fail] drawing co-link smoke:', error);
  process.exitCode = 1;
  setTimeout(() => process.exit(1), 500);
});
