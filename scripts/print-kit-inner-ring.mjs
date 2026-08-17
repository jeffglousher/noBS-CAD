#!/usr/bin/env node
/**
 * Retrofit inner keeper ring for the original printed VAWT stator
 * (race ID 48.5, no keepers). Also copies the latest kit into its own folder.
 *
 *   node scripts/print-kit-inner-ring.mjs
 */
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { strToU8, zipSync } from "fflate";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "..");
const documents = path.join(os.homedir(), "Documents", "noBS-CAD");
const latestDir = path.join(documents, "Print-Kit-Latest");
const ringDir = path.join(documents, "Print-Kit-Inner-Ring");

// Original printed stator (clip-era exam, before keeper walls).
const originalRaceId = 48.5;
const hubOd = 16.0;
const running = 0.4;
const bedLeadIn = 0.8;
const ringOd = originalRaceId - running;
const ringId = hubOd + bedLeadIn;
const ringH = 6.4;
const plaOrange = "bambu.pla.basic.orange";

function defaultBin() {
  if (process.env.NBCAD_MCP_BIN) return process.env.NBCAD_MCP_BIN;
  const names = [
    path.join(repoRoot, "mcp-server", "target", "debug", "nbcad-mcp.exe"),
    path.join(repoRoot, "mcp-server", "target", "release", "nbcad-mcp.exe"),
    path.join(repoRoot, "mcp-server", "target", "debug", "nbcad-mcp"),
    path.join(repoRoot, "mcp-server", "target", "release", "nbcad-mcp"),
  ];
  return names.find((candidate) => existsSync(candidate));
}

function writeNbcadArchive(modelJson, destination) {
  const model = JSON.parse(modelJson);
  if (model.format !== "nbcad-project" || !Number.isInteger(model.schema_version)) {
    throw new Error("cad_project_model did not return a .nbcad model.json payload");
  }
  const manifest = {
    format: "nbcad-project",
    container_version: 1,
    model: "model.json",
    application: "noBS CAD",
    application_version: "0.1.0",
    saved_at: new Date().toISOString(),
  };
  const bytes = zipSync(
    {
      "manifest.json": strToU8(`${JSON.stringify(manifest, null, 2)}\n`),
      "model.json": strToU8(modelJson.endsWith("\n") ? modelJson : `${modelJson}\n`),
    },
    { level: 6 },
  );
  mkdirSync(path.dirname(destination), { recursive: true });
  writeFileSync(destination, bytes);
  return bytes.length;
}

function copyLatestKit() {
  mkdirSync(latestDir, { recursive: true });
  const sources = [
    [path.join(documents, "Print-Kit-Tutor.nbcad"), path.join(latestDir, "Print-Kit-Latest.nbcad")],
    [path.join(documents, "Print-Kit-Tutor", "01-kit.3mf"), path.join(latestDir, "01-kit.3mf")],
    [path.join(documents, "Print-Kit-Tutor-design.md"), path.join(latestDir, "Print-Kit-Latest-design.md")],
  ];
  const copied = [];
  for (const [from, to] of sources) {
    if (!existsSync(from)) {
      throw new Error(`latest kit missing: ${from}`);
    }
    copyFileSync(from, to);
    copied.push(to);
  }
  return copied;
}

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
    "io.modelcontextprotocol/clientInfo": { name: "print-kit-inner-ring", version: "1" },
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
async function call(name, args = {}, timeoutMs = 120000) {
  return toolBody(await request("tools/call", { name, arguments: args }, timeoutMs));
}
function requireClean(update, label) {
  const errors = update?.scene?.errors ?? [];
  if (errors.length) throw new Error(`${label}: ${JSON.stringify(errors)}`);
  return update;
}
function lastSketch(doc) {
  return [...(doc.features ?? [])].reverse().find((feature) => feature.kind === "sketch")?.name;
}

async function buildRing() {
  await request("server/discover", {});
  await call("cad_new_project", {});
  await call("cad_set_document_name", { name: "Print Kit Inner Ring" });
  await call("cad_set_focus", { focus: "sketch", explicit: true });
  await call("sketch_begin", { plane: { type: "origin_plane", plane: "xy" } });
  await call("sketch_set_grid_snap", { enabled: false });
  await call("sketch_add_circle_locked", {
    mode: "center_diameter",
    anchor: { x: 0, y: 0 },
    edge_hint: { x: ringOd / 2, y: 0 },
    diameter_mm: ringOd,
    ctrl_held: false,
  });
  await call("sketch_add_circle_locked", {
    mode: "center_diameter",
    anchor: { x: 0, y: 0 },
    edge_hint: { x: ringId / 2, y: 0 },
    diameter_mm: ringId,
    ctrl_held: false,
  });
  const sketch = lastSketch(await call("cad_document"));
  await call("sketch_finish", {});
  const update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: ringH },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "inner keeper ring",
  );
  const bodyId = Math.max(...(update.scene?.bodies ?? []).map((body) => body.id));
  await call("cad_set_focus", { focus: "sketch", explicit: true });
  await call("sketch_begin", { plane: { type: "origin_plane", plane: "xy" } });
  await call("sketch_set_grid_snap", { enabled: false });
  await call("sketch_add_circle_locked", {
    mode: "center_diameter",
    anchor: { x: 0, y: 0 },
    edge_hint: { x: ringOd / 2, y: 0 },
    diameter_mm: ringOd,
    ctrl_held: false,
  });
  await call("sketch_add_circle_locked", {
    mode: "center_diameter",
    anchor: { x: 0, y: 0 },
    edge_hint: { x: (ringOd - 2 * bedLeadIn) / 2, y: 0 },
    diameter_mm: ringOd - 2 * bedLeadIn,
    ctrl_held: false,
  });
  const relief = lastSketch(await call("cad_document"));
  await call("sketch_finish", {});
  requireClean(
    await call("solid_extrude", {
      sketch_name: relief,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: bedLeadIn },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [bodyId],
    }),
    "ring OD lead-in",
  );
  await call("cad_set_focus", { focus: "print", explicit: true });
  await call("set_body_appearance", { body_id: bodyId, preset_id: plaOrange });
  const preflight = await call("solid_export_preflight");
  if (preflight.ok === false) {
    throw new Error(`preflight failed: ${JSON.stringify(preflight)}`);
  }
  mkdirSync(ringDir, { recursive: true });
  const project = await call("cad_project_model");
  const projectPath = path.join(ringDir, "Print-Kit-Inner-Ring.nbcad");
  writeNbcadArchive(
    typeof project === "string"
      ? project
      : typeof project?.model_json === "string"
        ? project.model_json
        : JSON.stringify(project),
    projectPath,
  );
  const exported = await call("solid_export_3mf", {
    slicer_target: "bambu_studio",
    include_appearance: true,
    body_ids: [bodyId],
  });
  const bytes = Buffer.from(exported.bytes_base64, "base64");
  const platePath = path.join(ringDir, "01-inner-ring.3mf");
  writeFileSync(platePath, bytes);
  writeFileSync(
    path.join(ringDir, "README.txt"),
    [
      "Print Kit inner keeper ring — retrofit for the original printed stator.",
      "",
      `Original race ID: ${originalRaceId.toFixed(1)} mm`,
      `Ring OD: ${ringOd.toFixed(1)} mm  (running −${running.toFixed(2)} so it drops in)`,
      `Ring ID: ${ringId.toFixed(1)} mm  (clears the Ø${hubOd.toFixed(1)} hub and Ø12 journal)`,
      `Height: ${ringH.toFixed(1)} mm`,
      `Lead-in: ${bedLeadIn.toFixed(1)} mm on the bed face`,
      "Material: PLA Basic Orange. Print flat.",
      "",
      "Install: rollers in the top-load slots, drop this ring over the journal",
      "from above, then drop the rotor on and snap the C-clip.",
      "The ring sits on the Y-frame ribs and blocks the roller inner ends.",
      "",
    ].join("\n"),
  );
  return { projectPath, platePath, bytes: bytes.length, bodyId };
}

const latest = copyLatestKit();
console.log("Latest kit:");
for (const file of latest) console.log(`  ${file}`);

try {
  const ring = await buildRing();
  console.log("Inner ring:");
  console.log(`  ${ring.projectPath}`);
  console.log(`  ${ring.platePath}  (${ring.bytes} bytes)`);
  console.log(
    `  OD ${ringOd.toFixed(1)} / ID ${ringId.toFixed(1)} / h ${ringH.toFixed(1)}  body ${ring.bodyId}`,
  );
} catch (error) {
  console.error(error);
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
