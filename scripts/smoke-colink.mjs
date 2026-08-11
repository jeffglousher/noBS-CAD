/**
 * Co-link smoke (browser path, no native OCCT required).
 *
 * 1. Start Vite with session HTTP bridge
 * 2. Warm WASM engine, publish a real UI session
 * 3. Simulate MCP writeback (bump generation + rename document)
 * 4. Assert UI applies the revision
 */
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { chromium } from 'playwright';

const PORT = 7198;
const ROOT = process.cwd();
const SESSION_DIR = fs.mkdtempSync(path.join(os.tmpdir(), 'nbcad-colink-'));
const BASE = `http://127.0.0.1:${PORT}`;

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
    const page = await browser.newPage();
    page.on('pageerror', (error) => {
      console.log('[colink] pageerror', error.message);
    });
    await page.goto(`${BASE}/`, { waitUntil: 'domcontentloaded' });
    await page.waitForFunction(
      () =>
        Boolean(window.__appStore) && typeof window.__nbcadPublishSession === 'function',
      null,
      { timeout: 60_000 },
    );

    console.log('[colink] warming engine');
    await page.evaluate(async () => {
      const { getEngine } = await import('/src/engine/index.ts');
      await getEngine();
    });

    console.log('[colink] publishing UI session');
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

    const model = JSON.parse(fs.readFileSync(modelPath, 'utf8'));
    const heartbeat = JSON.parse(fs.readFileSync(heartbeatPath, 'utf8'));
    const nextGeneration = Number(heartbeat.generation ?? 1) + 1;
    if (!model.document || typeof model.document !== 'object') {
      throw new Error('published model.json missing document object');
    }
    model.document.name = `colink-${nextGeneration}`;

    atomicWrite(
      writerPath,
      JSON.stringify({
        writer: 'mcp',
        updated_ms: Date.now(),
        generation: nextGeneration,
      }),
    );
    atomicWrite(modelPath, JSON.stringify(model));
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

    console.log('[colink] waiting for UI apply', nextGeneration);
    await page.waitForFunction(
      (expected) => window.__nbcadLastMcpGeneration === expected,
      nextGeneration,
      { timeout: 120_000 },
    );
    const docName = await page.evaluate(
      () => window.__appStore.getState().document?.name ?? null,
    );
    if (docName !== `colink-${nextGeneration}`) {
      throw new Error(`expected document name colink-${nextGeneration}, got ${docName}`);
    }

    console.log(`[ok] co-link smoke passed (session=${sessionId}, generation=${nextGeneration})`);
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
  console.error('[fail] co-link smoke:', error);
  process.exitCode = 1;
  setTimeout(() => process.exit(1), 500);
});
