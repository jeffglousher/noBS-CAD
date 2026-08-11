/**
 * Co-link smoke (browser path, no native OCCT required).
 *
 * 1. Start Vite with session HTTP bridge
 * 2. Open the app, wait for WASM engine
 * 3. Force UI session publish → UUID under NBCAD_SESSION_DIR
 * 4. Simulate MCP writeback: bump generation + source=mcp on model/heartbeat
 * 5. Assert UI applies the revision (__nbcadLastMcpGeneration)
 */
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { chromium } from 'playwright';

const PORT = 7198;
const ROOT = process.cwd();
const SESSION_DIR = fs.mkdtempSync(path.join(os.tmpdir(), 'nbcad-colink-'));

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
  const child = spawn(
    process.platform === 'win32' ? 'npm.cmd' : 'npm',
    ['run', 'dev', '--', '--host', '127.0.0.1', '--port', String(PORT), '--strictPort'],
    {
      cwd: ROOT,
      env: { ...process.env, NBCAD_SESSION_DIR: SESSION_DIR },
      stdio: ['ignore', 'pipe', 'pipe'],
    },
  );
  let serverLog = '';
  child.stdout.on('data', (chunk) => {
    serverLog += chunk.toString();
  });
  child.stderr.on('data', (chunk) => {
    serverLog += chunk.toString();
  });

  const browser = await chromium.launch({ headless: true });
  try {
    await waitForServer(`http://127.0.0.1:${PORT}/`);
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'domcontentloaded' });

    // Wait for app store + publish helper; force engine init via publish.
    await page.waitForFunction(
      () =>
        Boolean(window.__appStore) && typeof window.__nbcadPublishSession === 'function',
      null,
      { timeout: 60_000 },
    );

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
    // Marker the UI can observe after apply: bump nested document name.
    if (!model.document || typeof model.document !== 'object') {
      throw new Error('published model.json missing document object');
    }
    model.document.name = `colink-${nextGeneration}`;
    // Claim MCP writer first so the UI publisher will not overwrite the revision.
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

    await page.waitForFunction(
      (expected) => window.__nbcadLastMcpGeneration === expected,
      nextGeneration,
      { timeout: 30_000 },
    );
    const docName = await page.evaluate(
      () => window.__appStore.getState().document?.name ?? null,
    );
    if (docName !== `colink-${nextGeneration}`) {
      throw new Error(`expected document name colink-${nextGeneration}, got ${docName}`);
    }

    console.log(`[ok] co-link smoke passed (session=${sessionId}, generation=${nextGeneration})`);
  } finally {
    await browser.close();
    child.kill('SIGTERM');
    try {
      fs.rmSync(SESSION_DIR, { recursive: true, force: true });
    } catch {
      // best-effort
    }
    // Give vite a moment to exit; dump logs on failure path only via throw above.
    await sleep(500);
    if (serverLog.includes('error')) {
      // keep quiet on success
    }
  }
}

main().catch((error) => {
  console.error('[fail] co-link smoke:', error);
  process.exit(1);
});
