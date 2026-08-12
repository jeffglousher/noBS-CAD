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

const PREFIX = '/__nbcad_session';

function sessionRoot() {
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

function sendJson(res, status, body) {
  const payload = JSON.stringify(body);
  res.statusCode = status;
  res.setHeader('Content-Type', 'application/json');
  res.setHeader('Cache-Control', 'no-store');
  res.end(payload);
}

/** @type {Map<string, { sessionId: string, nextGeneration: number, lastApplied: number }>} */
const publishers = new Map();

function publisherFor(windowKey = 'browser') {
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

export function createSessionHttpBridge() {
  return {
    name: 'nbcad-session-http-bridge',
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        const url = new URL(req.url ?? '/', 'http://localhost');
        if (!url.pathname.startsWith(PREFIX)) {
          next();
          return;
        }

        try {
          const root = sessionRoot();
          const relative = url.pathname.slice(PREFIX.length).replace(/^\/+/, '');
          const parts = relative.split('/').filter(Boolean);

          if (req.method === 'POST' && parts[0] === 'reserve' && parts.length === 1) {
            const body = await readBody(req);
            const publisher = publisherFor(String(body.window ?? 'browser'));
            publisher.nextGeneration += 1;
            sendJson(res, 200, {
              session_id: publisher.sessionId,
              generation: publisher.nextGeneration,
              session_mode: 'live',
              session_dir: root,
            });
            return;
          }

          if (req.method === 'POST' && parts[0] === 'write' && parts.length === 1) {
            const body = await readBody(req);
            const publisher = publisherFor(String(body.window ?? 'browser'));
            const generation = Number(body.generation ?? 0);
            if (generation === 0 || generation > publisher.nextGeneration) {
              sendJson(res, 400, { error: `session generation ${generation} was not reserved` });
              return;
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
              return;
            }
            const dir = path.join(root, publisher.sessionId);
            fs.mkdirSync(dir, { recursive: true });
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
            atomicWrite(
              path.join(dir, 'writer.json'),
              JSON.stringify(
                {
                  writer: 'ui',
                  updated_ms: nowMs(),
                  generation,
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
            return;
          }

          if (req.method === 'POST' && parts[0] === 'heartbeat' && parts.length === 1) {
            const body = await readBody(req);
            const publisher = publisherFor(String(body.window ?? 'browser'));
            const dir = path.join(root, publisher.sessionId);
            if (!fs.existsSync(dir)) {
              sendJson(res, 200, {
                skipped: true,
                reason: 'no_session_dir',
                session_id: publisher.sessionId,
                session_mode: 'live',
              });
              return;
            }
            atomicWrite(
              path.join(dir, 'heartbeat.json'),
              JSON.stringify(
                {
                  updated_ms: nowMs(),
                  generation: publisher.lastApplied,
                  session_id: publisher.sessionId,
                  session_mode: 'live',
                  kind: 'heartbeat',
                  source: 'ui',
                },
                null,
                2,
              ),
            );
            sendJson(res, 200, {
              skipped: false,
              session_id: publisher.sessionId,
              generation: publisher.lastApplied,
              session_mode: 'live',
              writeback: true,
            });
            return;
          }

          if (parts.length === 2 && isValidSessionId(parts[0])) {
            const sessionId = parts[0];
            const file = parts[1];
            const dir = path.join(root, sessionId);
            if (req.method === 'GET') {
              if (!['model.json', 'focus.json', 'heartbeat.json', 'writer.json'].includes(file)) {
                sendJson(res, 400, { error: 'invalid filename' });
                return;
              }
              const filePath = path.join(dir, file);
              if (!fs.existsSync(filePath)) {
                sendJson(res, 404, { error: 'not found' });
                return;
              }
              if (file.endsWith('.json')) {
                sendJson(res, 200, readJson(filePath, {}));
              } else {
                res.statusCode = 200;
                res.end(fs.readFileSync(filePath));
              }
              return;
            }
            if (req.method === 'POST' && file === 'claim_writer') {
              const body = await readBody(req);
              const writer = String(body.writer ?? 'ui');
              const generation = Number(body.generation ?? 0);
              atomicWrite(
                path.join(dir, 'writer.json'),
                JSON.stringify(
                  { writer, updated_ms: nowMs(), generation },
                  null,
                  2,
                ),
              );
              sendJson(res, 200, { ok: true, writer, generation });
              return;
            }
            if (req.method === 'POST' && file === 'release_writer') {
              atomicWrite(
                path.join(dir, 'writer.json'),
                JSON.stringify(
                  { writer: 'none', updated_ms: nowMs(), generation: 0 },
                  null,
                  2,
                ),
              );
              sendJson(res, 200, { ok: true, writer: 'none' });
              return;
            }
          }

          sendJson(res, 404, { error: 'unknown session bridge route' });
        } catch (error) {
          sendJson(res, 500, {
            error: error instanceof Error ? error.message : String(error),
          });
        }
      });
    },
  };
}
