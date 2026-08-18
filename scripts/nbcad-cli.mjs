#!/usr/bin/env node
/**
 * Headless noBS CAD CLI — talk to nbcad-mcp without reloading Cursor MCP.
 *
 *   node scripts/nbcad-cli.mjs solid_check
 *   node scripts/nbcad-cli.mjs call solid_scene
 *   node scripts/nbcad-cli.mjs call solid_check '{"linear_deflection":0.2}'
 *   node scripts/nbcad-cli.mjs tools
 *   node scripts/nbcad-cli.mjs exam --stage=stator|rollers|clip|kit
 *
 * Requires native OCCT (OCCT_ROOT on PATH). Set NBCAD_MCP_BIN to pick a binary;
 * otherwise prefers mcp-server/target/debug then release. Rebuild after Rust
 * tool changes: cargo build --manifest-path mcp-server/Cargo.toml
 */
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "..");

function defaultBin() {
  if (process.env.NBCAD_MCP_BIN) return process.env.NBCAD_MCP_BIN;
  const names = [
    ["mcp-server", "target", "debug", "nbcad-mcp.exe"],
    ["mcp-server", "target", "debug", "nbcad-mcp"],
    ["mcp-server", "target", "release", "nbcad-mcp.exe"],
    ["mcp-server", "target", "release", "nbcad-mcp"],
  ];
  return names.map((parts) => path.join(repoRoot, ...parts)).find((candidate) => existsSync(candidate));
}

function printHelp() {
  console.log(`nbcad-cli — headless CAD without restarting Cursor MCP

Usage:
  node scripts/nbcad-cli.mjs solid_check [json-args]
  node scripts/nbcad-cli.mjs call <tool> [json-args]
  node scripts/nbcad-cli.mjs tools
  node scripts/nbcad-cli.mjs exam --stage=stator|rollers|clip|kit

Environment:
  NBCAD_MCP_BIN   path to nbcad-mcp (default: mcp-server/target/debug)
  OCCT_ROOT       native OCCT (required for solids)
`);
}

const argv = process.argv.slice(2);
if (argv.length === 0 || argv[0] === "-h" || argv[0] === "--help") {
  printHelp();
  process.exit(argv.length === 0 ? 2 : 0);
}

const command = argv[0];
if (command === "exam") {
  const tutor = path.join(here, "mcp-print-kit-tutor.mjs");
  const child = spawn(process.execPath, [tutor, ...argv.slice(1)], {
    env: process.env,
    stdio: "inherit",
  });
  child.on("exit", (code, signal) => {
    if (signal) process.exit(1);
    process.exit(code ?? 1);
  });
} else {
  await runMcp(command, argv.slice(1));
}

async function runMcp(command, rest) {
  const bin = defaultBin();
  if (!bin) {
    console.error("nbcad-mcp binary not found. Build mcp-server or set NBCAD_MCP_BIN.");
    process.exit(2);
  }

  const child = spawn(bin, [], { env: process.env, stdio: ["pipe", "pipe", "pipe"] });
  child.stderr.on("data", (chunk) => process.stderr.write(chunk));
  const pending = new Map();
  let nextId = 1;
  createInterface({ input: child.stdout }).on("line", (line) => {
    if (!line.trim()) return;
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      return;
    }
    if (message.id != null && pending.has(message.id)) {
      pending.get(message.id)(message);
      pending.delete(message.id);
    }
  });

  function meta() {
    return {
      "io.modelcontextprotocol/protocolVersion": "2026-07-28",
      "io.modelcontextprotocol/clientCapabilities": {},
      "io.modelcontextprotocol/clientInfo": { name: "nbcad-cli", version: "1" },
    };
  }
  function request(method, params = {}, timeoutMs = 90000) {
    const id = nextId++;
    const promise = new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        pending.delete(id);
        reject(new Error(`timeout ${method}`));
      }, timeoutMs);
      pending.set(id, (message) => {
        clearTimeout(timer);
        resolve(message);
      });
    });
    child.stdin.write(
      `${JSON.stringify({ jsonrpc: "2.0", id, method, params: { _meta: meta(), ...params } })}\n`,
    );
    return promise;
  }
  function toolBody(message) {
    if (message.error) throw new Error(JSON.stringify(message.error));
    const result = message.result;
    if (result?.isError) throw new Error(result.content?.[0]?.text ?? JSON.stringify(result));
    const structured = result?.structuredContent;
    if (structured && Object.prototype.hasOwnProperty.call(structured, "value")) return structured.value;
    if (structured && typeof structured === "object") {
      const { _protocol, _session, ...rest } = structured;
      if (Object.prototype.hasOwnProperty.call(rest, "result") && Object.keys(rest).length === 1) {
        return rest.result;
      }
      return Object.keys(rest).length ? rest : structured;
    }
    if (result?.content?.[0]?.text) {
      try {
        return JSON.parse(result.content[0].text);
      } catch {
        return result.content[0].text;
      }
    }
    return result;
  }

  try {
    await request("server/discover", {});
    let payload;
    if (command === "tools") {
      payload = toolBody(await request("tools/list", {}));
    } else if (command === "solid_check" || command === "call") {
      const tool = command === "solid_check" ? "solid_check" : rest[0];
      if (!tool) {
        console.error("call requires a tool name");
        process.exit(2);
      }
      const raw = command === "solid_check" ? rest[0] : rest[1];
      let args = {};
      if (raw) {
        args = JSON.parse(raw);
      }
      payload = toolBody(await request("tools/call", { name: tool, arguments: args }));
    } else {
      console.error(`unknown command: ${command}`);
      printHelp();
      process.exit(2);
    }
    console.log(JSON.stringify(payload, null, 2));
    if (command === "solid_check" && payload && payload.ok === false) {
      process.exitCode = 1;
    }
  } catch (error) {
    console.error(String(error?.stack ?? error));
    process.exitCode = 1;
  } finally {
    try {
      child.stdin.end();
    } catch {
      /* ignore */
    }
    setTimeout(() => {
      if (!child.killed) child.kill();
      process.exit(process.exitCode ?? 0);
    }, 400);
  }
}
