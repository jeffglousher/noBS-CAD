#!/usr/bin/env node
/**
 * Demo: drive nbcad-mcp over bus-jsonl (the Kafka/MQTT/NATS integration shape).
 *
 * Usage:
 *   NBCAD_MCP_BIN=/path/to/nbcad-mcp node scripts/mcp-bus-jsonl-demo.mjs
 *
 * For a live NATS broker, point a small connector at the same BusMessage
 * envelope (see docs/mcp-message-bus.md). This script proves the CAD-side
 * contract without embedding a broker SDK.
 */

import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { randomUUID } from 'node:crypto';

const bin = process.env.NBCAD_MCP_BIN;
if (!bin) {
  console.error('Set NBCAD_MCP_BIN to the nbcad-mcp executable');
  process.exit(2);
}

const documentId = process.env.NBCAD_DOCUMENT_ID ?? '00000000-0000-4000-8000-000000000001';
const child = spawn(bin, [], {
  env: { ...process.env, NBCAD_MCP_TRANSPORT: 'bus-jsonl' },
  stdio: ['pipe', 'pipe', 'inherit'],
});

const rl = createInterface({ input: child.stdout });
const pending = new Map();

rl.on('line', (line) => {
  if (!line.trim()) return;
  let message;
  try {
    message = JSON.parse(line);
  } catch (error) {
    console.error('bad bus frame from mcp:', error);
    return;
  }
  const wait = pending.get(message.correlation_id);
  if (wait) {
    pending.delete(message.correlation_id);
    wait.resolve(message);
  } else {
    console.log('notify/unsolicited', message.subject, message.payload?.toString?.() ?? message);
  }
});

function request(method, params = {}, id = 1) {
  const correlation = randomUUID();
  const payload = JSON.stringify({ jsonrpc: '2.0', id, method, params });
  const frame = {
    subject: `nbcad.mcp.${documentId}.req`,
    correlation_id: correlation,
    reply_to: `nbcad.mcp.${documentId}.res.${correlation}`,
    headers: {
      schema: 'nbcad.mcp-bus.v1',
      document_id: documentId,
      protocol_version: '2025-06-18',
    },
    payload,
  };
  const promise = new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(correlation);
      reject(new Error(`timeout waiting for ${method}`));
    }, 15000);
    pending.set(correlation, {
      resolve: (message) => {
        clearTimeout(timer);
        resolve(message);
      },
    });
  });
  child.stdin.write(`${JSON.stringify(frame)}\n`);
  return promise;
}

function payloadJson(message) {
  const raw = message.payload;
  if (typeof raw === 'string') {
    return JSON.parse(raw);
  }
  return JSON.parse(Buffer.from(Uint8Array.from(raw)).toString('utf8'));
}

const init = await request('initialize', { protocolVersion: '2025-06-18' }, 1);
console.log('initialize =>', payloadJson(init).result?.protocolVersion);

const listed = await request(
  'tools/call',
  { name: 'cad_list_sessions', arguments: {} },
  2,
);
console.log('cad_list_sessions =>', payloadJson(listed).result?.isError === false ? 'ok' : payloadJson(listed));

child.stdin.end();
child.on('exit', (code) => process.exit(code ?? 0));
