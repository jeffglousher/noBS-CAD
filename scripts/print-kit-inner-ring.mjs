#!/usr/bin/env node
/**
 * Retrofit inner keeper ring for the original printed VAWT stator
 * (race ID 48.5, no keepers). Also copies the latest kit into its own folder.
 *
 * Thin 2-pass slip hoop. Smooth OD. Three short Y-tabs so you can pick it
 * up. Not a washer. Not organic.
 *
 *   node scripts/print-kit-inner-ring.mjs
 */
import { spawn, spawnSync } from "node:child_process";
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

const originalRaceId = 48.5;
const hubOd = 16.0;
const journalOd = 12.0;
const nozzle = 0.4;
const wall = 2 * nozzle;

// Assembled FDM slip. 0.40 printed as a press.
const slip = 1.3;
const od = originalRaceId - slip; // 47.2
const id = od - 2 * wall; // 45.6 — the ring IS the 2-pass hoop
const hoopH = 3.2;
const tabInnerR = 14.4;
const tabOuterR = od / 2 - wall / 2; // land in the hoop wall
const padD = 3.2;
const plaOrange = "bambu.pla.basic.orange";

function polar(r, deg) {
  const a = (deg * Math.PI) / 180;
  return { x: r * Math.cos(a), y: r * Math.sin(a) };
}

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
  const missing = [];
  for (const [from, to] of sources) {
    if (!existsSync(from)) {
      missing.push(from);
      continue;
    }
    copyFileSync(from, to);
    copied.push(to);
  }
  return { copied, missing };
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

async function beginXY() {
  await call("cad_set_focus", { focus: "sketch", explicit: true });
  await call("sketch_begin", { plane: { type: "origin_plane", plane: "xy" } });
  await call("sketch_set_grid_snap", { enabled: false });
}
async function finishSketch() {
  await call("sketch_finish");
  return lastSketch(await call("cad_document"));
}
async function addCircle(x, y, diameter) {
  await call("sketch_add_circle_locked", {
    mode: "center_diameter",
    anchor: { x, y },
    edge_hint: { x: x + diameter / 2, y },
    diameter_mm: diameter,
    ctrl_held: false,
  });
}
async function addPoly(points, ctrlHeld = false) {
  for (let i = 0; i < points.length - 1; i++) {
    const dx = points[i + 1].x - points[i].x;
    const dy = points[i + 1].y - points[i].y;
    if (dx * dx + dy * dy < 1e-8) continue;
    await call("sketch_add_line", { from: points[i], to_raw: points[i + 1], ctrl_held: ctrlHeld });
  }
}
async function addOrientedRect(center, length, width, angleDeg) {
  const angle = (angleDeg * Math.PI) / 180;
  const ux = [Math.cos(angle), Math.sin(angle)];
  const uy = [-Math.sin(angle), Math.cos(angle)];
  const corner = (s, t) => ({
    x: center[0] + ux[0] * s + uy[0] * t,
    y: center[1] + ux[1] * s + uy[1] * t,
  });
  const hl = length / 2;
  const hw = width / 2;
  await addPoly(
    [corner(hl, hw), corner(hl, -hw), corner(-hl, -hw), corner(-hl, hw), corner(hl, hw)],
    true,
  );
}
async function extrude(sketch, operation, distance, bodyIds, label) {
  return requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation,
      extent: { type: "distance", distance },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: bodyIds,
    }),
    label,
  );
}

async function buildRing() {
  await request("server/discover", {});
  await call("cad_new_project", {});
  await call("cad_set_document_name", { name: "Print Kit Inner Ring" });

  await beginXY();
  await addCircle(0, 0, od);
  await addCircle(0, 0, id);
  const hoop = await finishSketch();
  const created = await extrude(hoop, "new_body", hoopH, [], "slip hoop");
  const bodyId = Math.max(...(created.scene?.bodies ?? []).map((body) => body.id));

  for (let i = 0; i < 3; i++) {
    const angle = i * 120;
    await beginXY();
    const mid = (tabInnerR + tabOuterR) / 2;
    const p = polar(mid, angle);
    await addOrientedRect([p.x, p.y], tabOuterR - tabInnerR, wall, angle);
    const tab = await finishSketch();
    await extrude(tab, "join", hoopH, [bodyId], `tab ${i}`);
    const tip = polar(tabInnerR, angle);
    await beginXY();
    await addCircle(tip.x, tip.y, padD);
    const pad = await finishSketch();
    await extrude(pad, "join", hoopH, [bodyId], `pad ${i}`);
  }

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
  const readme = [
    "Print Kit inner keeper ring — retrofit for the original printed stator.",
    "",
    "Thin 2-pass slip hoop. Not a washer. Not organic.",
    "",
    `Original race ID: ${originalRaceId.toFixed(1)} mm`,
    `Hoop OD: ${od.toFixed(1)} mm   (assembled FDM slip −${slip.toFixed(2)})`,
    `Hoop ID: ${id.toFixed(1)} mm   (2-pass wall ${wall.toFixed(1)} mm)`,
    `Height: ${hoopH.toFixed(1)} mm`,
    `Tabs: 3 at 120°, ${wall.toFixed(1)} mm, pads Ø${padD.toFixed(1)} — pickup only`,
    `Clears the Ø${hubOd.toFixed(1)} hub and Ø${journalOd.toFixed(1)} journal by a lot.`,
    "Material: PLA Basic Orange. Print flat on the X2D, 0.4 mm nozzle.",
    "Suggested: 0.20 mm layer, 2 walls, no supports, no brim.",
    "",
    "Drop rollers into the top-load slots, then drop this hoop over the",
    "journal. It sits on the Y-frame ribs and blocks the roller inner ends",
    "without gripping the race. Rotor on, C-clip on.",
    "",
  ].join("\n");
  writeFileSync(path.join(ringDir, "README.txt"), readme);
  writeFileSync(path.join(ringDir, "Print-Kit-Inner-Ring-design.md"), `# Inner keeper ring\n\n${readme}`);
  const preview = spawnSync(process.execPath, [path.join(here, "preview-inner-ring.mjs")], {
    stdio: "inherit",
  });
  if (preview.status) {
    throw new Error(`preview-inner-ring exited ${preview.status}`);
  }
  return { projectPath, platePath, bytes: bytes.length, bodyId, preflight };
}

const latest = copyLatestKit();
if (latest.copied.length) {
  console.log("Latest kit:");
  for (const file of latest.copied) console.log(`  ${file}`);
}
if (latest.missing.length) {
  console.log("Latest kit skipped (missing sources):");
  for (const file of latest.missing) console.log(`  ${file}`);
}

try {
  const ring = await buildRing();
  console.log("Inner ring:");
  console.log(`  ${ring.projectPath}`);
  console.log(`  ${ring.platePath}  (${ring.bytes} bytes)`);
  console.log(
    `  OD ${od.toFixed(1)} / ID ${id.toFixed(1)} / h ${hoopH.toFixed(1)} / wall ${wall.toFixed(1)}  body ${ring.bodyId}`,
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
