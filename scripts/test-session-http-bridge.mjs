/**
 * Bidirectional session-bridge tests (no WASM / OCCT).
 *
 * Covers UI publish → MCP takeover files → MCP writeback → UI heartbeat
 * must not clobber the MCP revision.
 */
import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  PREFIX,
  createPublisherMap,
  dispatchSessionRequest,
} from './session-http-bridge.mjs';
import { Domain, mint } from './nbcad-id.mjs';

const ROOT = fs.mkdtempSync(path.join(os.tmpdir(), 'nbcad-bridge-test-'));
process.env.NBCAD_SESSION_DIR = ROOT;

function listen(server) {
  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => resolve(server.address().port));
  });
}

function close(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
}

async function jsonRequest(port, method, route, body) {
  const response = await fetch(`http://127.0.0.1:${port}${PREFIX}${route}`, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const payload = await response.json();
  return { status: response.status, payload };
}

function atomicWrite(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const temporary = `${filePath}.${process.pid}.tmp`;
  fs.writeFileSync(temporary, content, 'utf8');
  fs.renameSync(temporary, filePath);
}

function simulateMcpWriteback(sessionId, model, generation) {
  const dir = path.join(ROOT, sessionId);
  atomicWrite(path.join(dir, 'model.json'), JSON.stringify(model));
  atomicWrite(
    path.join(dir, 'heartbeat.json'),
    JSON.stringify({
      updated_ms: Date.now(),
      generation,
      session_id: sessionId,
      session_mode: 'live',
      source: 'mcp',
    }),
  );
  atomicWrite(
    path.join(dir, 'writer.json'),
    JSON.stringify({ writer: 'mcp', updated_ms: Date.now(), generation }),
  );
}

async function main() {
  const publishers = createPublisherMap();
  const server = http.createServer((req, res) => {
    void dispatchSessionRequest(req, res, () => {
      res.statusCode = 404;
      res.end('not bridge');
    }, { publishers, root: ROOT }).catch((error) => {
      res.statusCode = 500;
      res.end(String(error));
    });
  });
  const port = await listen(server);

  try {
    const reserved = await jsonRequest(port, 'POST', '/reserve', { window: 'browser' });
    assert.equal(reserved.status, 200);
    const sessionId = reserved.payload.session_id;
    const generation = reserved.payload.generation;
    assert.equal(typeof sessionId, 'string');
    assert.equal(sessionId.length, 36);
    assert.equal(generation, 1);
    assert.equal(reserved.payload.session_mode, 'live');

    const written = await jsonRequest(port, 'POST', '/write', {
      window: 'browser',
      generation,
      focus: 'solid',
      model_json: { document: { name: 'FromUi' }, version: 1 },
    });
    assert.equal(written.status, 200);
    assert.equal(written.payload.skipped, false);
    assert.equal(written.payload.session_id, sessionId);

    const model = await jsonRequest(port, 'GET', `/${sessionId}/model.json`);
    assert.equal(model.status, 200);
    assert.equal(model.payload.document.name, 'FromUi');

    const writer = await jsonRequest(port, 'GET', `/${sessionId}/writer.json`);
    assert.equal(writer.payload.writer, 'ui');
    assert.equal(writer.payload.generation, 1);

    const heartbeat = await jsonRequest(port, 'GET', `/${sessionId}/heartbeat.json`);
    assert.equal(heartbeat.payload.source, 'ui');
    assert.equal(heartbeat.payload.generation, 1);

    const staleWrite = await jsonRequest(port, 'POST', '/write', {
      window: 'browser',
      generation: 1,
      focus: 'solid',
      model_json: { document: { name: 'Stale' } },
    });
    assert.equal(staleWrite.payload.skipped, true);
    assert.equal(staleWrite.payload.reason, 'stale_generation');

    const unreserved = await jsonRequest(port, 'POST', '/write', {
      window: 'browser',
      generation: 99,
      focus: 'solid',
      model_json: {},
    });
    assert.equal(unreserved.status, 400);

    // MCP live-attach takeover + writeback (file contract, no OCCT).
    simulateMcpWriteback(
      sessionId,
      { document: { name: 'FromMcp' }, version: 1 },
      2,
    );
    const afterMcp = await jsonRequest(port, 'GET', `/${sessionId}/model.json`);
    assert.equal(afterMcp.payload.document.name, 'FromMcp');
    assert.equal((await jsonRequest(port, 'GET', `/${sessionId}/writer.json`)).payload.writer, 'mcp');

    const beat = await jsonRequest(port, 'POST', '/heartbeat', { window: 'browser' });
    assert.equal(beat.status, 200);
    assert.equal(beat.payload.preserved_mcp, true);
    assert.equal(beat.payload.source, 'mcp');
    assert.equal(beat.payload.generation, 2);
    const beatFile = await jsonRequest(port, 'GET', `/${sessionId}/heartbeat.json`);
    assert.equal(beatFile.payload.source, 'mcp');
    assert.equal(beatFile.payload.generation, 2);
    assert.equal(
      (await jsonRequest(port, 'GET', `/${sessionId}/model.json`)).payload.document.name,
      'FromMcp',
      'UI heartbeat must not clobber MCP model.json',
    );

    const next = await jsonRequest(port, 'POST', '/reserve', { window: 'browser' });
    const skippedSteal = await jsonRequest(port, 'POST', '/write', {
      window: 'browser',
      generation: next.payload.generation,
      focus: 'document',
      model_json: { document: { name: 'ShouldNotSteal' } },
    });
    assert.equal(skippedSteal.payload.skipped, true);
    assert.equal(skippedSteal.payload.reason, 'writer_held_by_mcp');
    assert.equal(
      (await jsonRequest(port, 'GET', `/${sessionId}/model.json`)).payload.document.name,
      'FromMcp',
    );

    const conflict = await jsonRequest(port, 'POST', `/${sessionId}/claim_writer`, {
      writer: 'ui',
      generation: 3,
    });
    assert.equal(conflict.status, 409);
    assert.match(conflict.payload.error, /session writer conflict/);

    const released = await jsonRequest(port, 'POST', `/${sessionId}/release_writer`);
    assert.equal(released.status, 200);
    assert.equal(released.payload.writer, 'none');
    assert.equal(released.payload.generation, 2);

    const reclaim = await jsonRequest(port, 'POST', `/${sessionId}/claim_writer`, {
      writer: 'ui',
      generation: 3,
    });
    assert.equal(reclaim.status, 200);
    assert.equal(reclaim.payload.writer, 'ui');

    const badFile = await jsonRequest(port, 'GET', `/${sessionId}/secret.txt`);
    assert.equal(badFile.status, 400);

    const missing = await jsonRequest(port, 'GET', `/${mint(Domain.Session)}/model.json`);
    assert.equal(missing.status, 404);

    const unknown = await jsonRequest(port, 'POST', '/nope', {});
    assert.equal(unknown.status, 404);

    const windowA = await jsonRequest(port, 'POST', '/reserve', { window: 'window-a' });
    const windowB = await jsonRequest(port, 'POST', '/reserve', { window: 'window-b' });
    assert.notEqual(windowA.payload.session_id, windowB.payload.session_id);
    assert.equal(
      (await jsonRequest(port, 'POST', '/write', {
        window: 'window-a',
        generation: windowA.payload.generation,
        focus: 'solid',
        model_json: { document: { name: 'WindowA' } },
      })).status,
      200,
    );
    assert.equal(
      (await jsonRequest(port, 'POST', '/write', {
        window: 'window-b',
        generation: windowB.payload.generation,
        focus: 'solid',
        model_json: { document: { name: 'WindowB' } },
      })).status,
      200,
    );
    assert.equal(
      (await jsonRequest(port, 'GET', `/${windowA.payload.session_id}/model.json`)).payload.document
        .name,
      'WindowA',
    );
    assert.equal(
      (await jsonRequest(port, 'GET', `/${windowB.payload.session_id}/model.json`)).payload.document
        .name,
      'WindowB',
    );
  } finally {
    await close(server);
    fs.rmSync(ROOT, { recursive: true, force: true });
  }

  console.log('[ok] session HTTP bridge bidirectional tests passed');
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  main().catch((error) => {
    console.error('[fail] session HTTP bridge tests:', error);
    process.exitCode = 1;
  });
}
