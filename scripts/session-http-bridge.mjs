/**
 * Vite middleware: browser UI ↔ NBCAD_SESSION_DIR for MCP co-link.
 *
 * Mirrors Tauri session_bridge reserve/write/heartbeat so WASM/cloud agents
 * can publish and poll live sessions without a desktop shell.
 *
 * Routes (dev server only):
 *   POST /__nbcad_session/reserve
 *   POST /__nbcad_session/write
 *   POST /__nbcad_session/heartbeat
 *   GET  /__nbcad_session/:uuid/:file
 *   POST /__nbcad_session/:uuid/claim_writer
 *   POST /__nbcad_session/:uuid/release_writer
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { Domain, isValidSessionId, mint } from './nbcad-id.mjs';

export const PREFIX = '/__nbcad_session';
const SESSION_FILES = ['model.json', 'focus.json', 'heartbeat.json', 'writer.json', 'window.json'];

export function sessionRoot() {
  const custom = process.env.NBCAD_SESSION_DIR?.trim();
  return custom && custom.length > 0
    ? custom
    : path.join(os.tmpdir(), 'nbcad-sessions');
}

function nowMs() {
  return Date.now();
}

function atomicWrite(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const temporary = `${filePath}.${process.pid}.tmp`;
  fs.writeFileSync(temporary, content, 'utf8');
  fs.renameSync(temporary, filePath);
}

function readJson(filePath, fallback = null) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch {
    return fallback;
  }
}

function readWriter(dir) {
  const parsed = readJson(path.join(dir, 'writer.json'), {});
  const writer = parsed?.writer;
  if (writer === 'ui' || writer === 'mcp' || writer === 'none') {
    return {
      writer,
      updated_ms: Number(parsed.updated_ms ?? 0),
      generation: Number(parsed.generation ?? 0),
    };
  }
  return { writer: 'none', updated_ms: 0, generation: 0 };
}

function writeWriter(dir, writer, generation) {
  atomicWrite(
    path.join(dir, 'writer.json'),
    JSON.stringify({ writer, updated_ms: nowMs(), generation }, null, 2),
  );
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', (chunk) => chunks.push(chunk));
    req.on('end', () => {
      const raw = Buffer.concat(chunks).toString('utf8');
      if (!raw) {
        resolve({});
        return;
      }
      try {
        resolve(JSON.parse(raw));
      } catch (error) {
        reject(error);
      }
    });
    req.on('error', reject);
  });
}

export function sendJson(res, status, body) {
  const payload = JSON.stringify(body);
  res.statusCode = status;
  res.setHeader('Content-Type', 'application/json');
  res.setHeader('Cache-Control', 'no-store');
  res.end(payload);
}

/** @type {Map<string, { sessionId: string, nextGeneration: number, lastApplied: number }>} */
export function createPublisherMap() {
  return new Map();
}

function publisherFor(publishers, windowKey = 'browser') {
  let publisher = publishers.get(windowKey);
  if (!publisher) {
    publisher = {
      sessionId: mint(Domain.Session),
      nextGeneration: 0,
      lastApplied: 0,
    };
    publishers.set(windowKey, publisher);
  }
  return publisher;
}

/**
 * Handle one `/__nbcad_session` request. Returns true if the request was
 * consumed. `next` is called only when the path is outside the prefix.
 */
export async function dispatchSessionRequest(req, res, next, ctx = {}) {
  const publishers = ctx.publishers ?? createPublisherMap();
  const root = ctx.root ?? sessionRoot();
  const url = new URL(req.url ?? '/', 'http://localhost');
  if (!url.pathname.startsWith(PREFIX)) {
    next();
    return false;
  }

  const relative = url.pathname.slice(PREFIX.length).replace(/^\/+/, '');
  const parts = relative.split('/').filter(Boolean);

  if (req.method === 'POST' && parts[0] === 'reserve' && parts.length === 1) {
    const body = await readBody(req);
    const publisher = publisherFor(publishers, String(body.window ?? 'browser'));
    publisher.nextGeneration += 1;
    sendJson(res, 200, {
      session_id: publisher.sessionId,
      generation: publisher.nextGeneration,
      session_mode: 'live',
      session_dir: root,
    });
    return true;
  }

  if (req.method === 'POST' && parts[0] === 'write' && parts.length === 1) {
    const body = await readBody(req);
    const publisher = publisherFor(publishers, String(body.window ?? 'browser'));
    const generation = Number(body.generation ?? 0);
    if (generation === 0 || generation > publisher.nextGeneration) {
      sendJson(res, 400, { error: `session generation ${generation} was not reserved` });
      return true;
    }
    if (generation <= publisher.lastApplied) {
      sendJson(res, 200, {
        skipped: true,
        reason: 'stale_generation',
        session_id: publisher.sessionId,
        generation,
        last_applied_generation: publisher.lastApplied,
        session_mode: 'live',
      });
      return true;
    }
    const dir = path.join(root, publisher.sessionId);
    fs.mkdirSync(dir, { recursive: true });
    const holder = readWriter(dir);
    if (holder.writer === 'mcp') {
      // MCP is the live writer. UI keeps displaying MCP revisions; do not steal.
      sendJson(res, 200, {
        skipped: true,
        reason: 'writer_held_by_mcp',
        session_id: publisher.sessionId,
        generation,
        writer: holder,
        session_mode: 'live',
      });
      return true;
    }
    const focus = String(body.focus ?? 'document');
    const modelJson =
      typeof body.model_json === 'string'
        ? body.model_json
        : JSON.stringify(body.model_json ?? {});
    atomicWrite(
      path.join(dir, 'focus.json'),
      JSON.stringify(
        {
          focus,
          session_id: publisher.sessionId,
          updated_ms: nowMs(),
          generation,
          session_mode: 'live',
        },
        null,
        2,
      ),
    );
    atomicWrite(
      path.join(dir, 'heartbeat.json'),
      JSON.stringify(
        {
          updated_ms: nowMs(),
          generation,
          session_id: publisher.sessionId,
          session_mode: 'live',
          source: 'ui',
        },
        null,
        2,
      ),
    );
    atomicWrite(path.join(dir, 'model.json'), modelJson);
    writeWriter(dir, 'ui', generation);
    atomicWrite(
      path.join(dir, 'window.json'),
      JSON.stringify(
        {
          window: String(body.window ?? 'browser'),
          session_id: publisher.sessionId,
          session_mode: 'live',
        },
        null,
        2,
      ),
    );
    publisher.lastApplied = generation;
    sendJson(res, 200, {
      skipped: false,
      session_id: publisher.sessionId,
      session_dir: dir,
      generation,
      session_mode: 'live',
      writeback: true,
    });
    return true;
  }

  if (req.method === 'POST' && parts[0] === 'heartbeat' && parts.length === 1) {
    const body = await readBody(req);
    const publisher = publisherFor(publishers, String(body.window ?? 'browser'));
    const dir = path.join(root, publisher.sessionId);
    if (!fs.existsSync(dir)) {
      sendJson(res, 200, {
        skipped: true,
        reason: 'no_session_dir',
        session_id: publisher.sessionId,
        session_mode: 'live',
      });
      return true;
    }
    const holder = readWriter(dir);
    const existing = readJson(path.join(dir, 'heartbeat.json'), {});
    const mcpHolds =
      holder.writer === 'mcp' ||
      (existing?.source === 'mcp' && Number(existing.generation ?? 0) > publisher.lastApplied);
    const generation = mcpHolds
      ? Number(existing.generation ?? publisher.lastApplied)
      : publisher.lastApplied;
    const source = mcpHolds ? String(existing.source ?? 'mcp') : 'ui';
    atomicWrite(
      path.join(dir, 'heartbeat.json'),
      JSON.stringify(
        {
          updated_ms: nowMs(),
          generation,
          session_id: publisher.sessionId,
          session_mode: 'live',
          kind: 'heartbeat',
          source,
        },
        null,
        2,
      ),
    );
    sendJson(res, 200, {
      skipped: false,
      session_id: publisher.sessionId,
      generation,
      source,
      preserved_mcp: mcpHolds,
      session_mode: 'live',
      writeback: true,
    });
    return true;
  }

  if (parts.length === 2 && isValidSessionId(parts[0])) {
    const sessionId = parts[0];
    const file = parts[1];
    const dir = path.join(root, sessionId);
    if (req.method === 'GET') {
      if (!SESSION_FILES.includes(file)) {
        sendJson(res, 400, { error: 'invalid filename' });
        return true;
      }
      const filePath = path.join(dir, file);
      if (!fs.existsSync(filePath)) {
        sendJson(res, 404, { error: 'not found' });
        return true;
      }
      sendJson(res, 200, readJson(filePath, {}));
      return true;
    }
    if (req.method === 'POST' && file === 'claim_writer') {
      const body = await readBody(req);
      const writer = String(body.writer ?? 'ui');
      if (!['ui', 'mcp', 'none'].includes(writer)) {
        sendJson(res, 400, { error: `writer must be 'ui', 'mcp', or 'none' (got '${writer}')` });
        return true;
      }
      fs.mkdirSync(dir, { recursive: true });
      const generation = Number(body.generation ?? 0);
      const holder = readWriter(dir);
      if (holder.writer !== 'none' && holder.writer !== writer) {
        sendJson(res, 409, {
          error: `session writer conflict: ${holder.writer} holds the writer lock; call cad_refresh or wait`,
          writer: holder,
        });
        return true;
      }
      writeWriter(dir, writer, generation);
      sendJson(res, 200, { ok: true, writer, generation });
      return true;
    }
    if (req.method === 'POST' && file === 'release_writer') {
      const holder = readWriter(dir);
      writeWriter(dir, 'none', holder.generation);
      sendJson(res, 200, { ok: true, writer: 'none', generation: holder.generation });
      return true;
    }
  }

  sendJson(res, 404, { error: 'unknown session bridge route' });
  return true;
}

export function createSessionHttpBridge() {
  const publishers = createPublisherMap();
  return {
    name: 'nbcad-session-http-bridge',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        void dispatchSessionRequest(req, res, next, { publishers }).catch((error) => {
          if (!res.headersSent) {
            sendJson(res, 500, {
              error: error instanceof Error ? error.message : String(error),
            });
          }
        });
      });
    },
  };
}
