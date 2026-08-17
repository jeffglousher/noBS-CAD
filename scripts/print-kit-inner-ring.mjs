#!/usr/bin/env node
/**
 * Retrofit inner keeper ring for the original printed VAWT stator
 * (race ID 48.5, no keepers). Also copies the latest kit into its own folder.
 *
 * Slip-fit jeweled hoop — not a friction washer. Smooth circular OD/ID
 * lands only; the web is an aired-out trillium of 2-pass bones for a
 * 0.4 mm nozzle on the X2D.
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
const journalOd = 12.0;

// X2D / 0.4 mm nozzle. 2-pass = 0.8, 4-pass land = 1.6.
const nozzle = 0.4;
const wall2 = 2 * nozzle;
const wall4 = 4 * nozzle;

// First ring used running −0.40 on the OD and printed as a friction plug.
// FDM ODs grow and printed holes shrink; assembled slip needs real air.
const slip = 1.3;
const od = originalRaceId - slip; // 47.2
const id = 21.6; // clears the Ø16 hub — not a fit, just a smooth bore
const land = wall4;
const odInner = od - 2 * land; // 44.0
const idOuter = id + 2 * land; // 24.8

const hoopH = 6.4;
const petalH = 5.2;
const vineH = 4.8;
const leadH = 1.6;
const leadRadial = 0.8;
const leadOd = od - 2 * leadRadial; // 45.6
const seedD = 3.2;
const plaOrange = "bambu.pla.basic.orange";

// Swirling trillium (120°) plus opposite-hand 2-pass tendrils (60°).
const petalStations = [
  { r: 13.6, da: 2, d: 5.0 },
  { r: 14.8, da: 8, d: 6.6 },
  { r: 16.2, da: 13, d: 8.2 },
  { r: 17.6, da: 16, d: 9.2 },
  { r: 19.0, da: 12, d: 7.8 },
  { r: 20.2, da: 6, d: 6.0 },
  { r: 21.3, da: 1, d: 4.4 },
];
const petalLobes = [
  { r: 16.8, da: 30, d: 5.6 },
  { r: 18.2, da: -6, d: 6.8 },
];
const vineStations = [
  { r: 13.2, da: 0, d: 2.6 },
  { r: 14.8, da: -5, d: 3.0 },
  { r: 16.4, da: -9, d: 3.2 },
  { r: 18.0, da: -11, d: 3.2 },
  { r: 19.6, da: -7, d: 2.8 },
  { r: 21.2, da: -2, d: 2.4 },
];

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
async function addRadialBone(angleDeg, innerR, outerR, width) {
  const mid = (innerR + outerR) / 2;
  const p = polar(mid, angleDeg);
  await addOrientedRect([p.x, p.y], outerR - innerR, width, angleDeg);
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
async function annulus(outerD, innerD) {
  await beginXY();
  await addCircle(0, 0, outerD);
  await addCircle(0, 0, innerD);
  return finishSketch();
}
async function cutCircle(x, y, diameter, depth, bodyId, label) {
  await beginXY();
  await addCircle(x, y, diameter);
  const sketch = await finishSketch();
  await extrude(sketch, "cut", depth, [bodyId], label);
}
async function joinCircle(x, y, diameter, height, bodyId, label) {
  await beginXY();
  await addCircle(x, y, diameter);
  const sketch = await finishSketch();
  await extrude(sketch, "join", height, [bodyId], label);
}
async function joinRadialBone(angleDeg, innerR, outerR, width, height, bodyId, label) {
  await beginXY();
  await addRadialBone(angleDeg, innerR, outerR, width);
  const sketch = await finishSketch();
  await extrude(sketch, "join", height, [bodyId], label);
}

async function buildRing() {
  await request("server/discover", {});
  await call("cad_new_project", {});
  await call("cad_set_document_name", { name: "Print Kit Inner Ring" });

  const outer = await annulus(od, odInner);
  const created = await extrude(outer, "new_body", hoopH, [], "outer land");
  const bodyId = Math.max(...(created.scene?.bodies ?? []).map((body) => body.id));

  const inner = await annulus(idOuter, id);
  await extrude(inner, "join", hoopH, [bodyId], "inner land");

  for (let i = 0; i < 3; i++) {
    const base = i * 120;
    let n = 0;
    for (const s of [...petalStations, ...petalLobes]) {
      const p = polar(s.r, base + s.da);
      await joinCircle(p.x, p.y, s.d, petalH, bodyId, `petal ${i}.${n}`);
      n += 1;
    }
  }

  for (let i = 0; i < 3; i++) {
    const base = 60 + i * 120;
    await joinRadialBone(base, idOuter / 2, odInner / 2, wall2, vineH, bodyId, `vine bone ${i}`);
    let n = 0;
    for (const s of vineStations) {
      const p = polar(s.r, base + s.da);
      await joinCircle(p.x, p.y, s.d, vineH, bodyId, `vine ${i}.${n}`);
      n += 1;
    }
  }

  const lead = await annulus(od, leadOd);
  await extrude(lead, "cut", leadH, [bodyId], "OD slip funnel");

  for (let i = 0; i < 3; i++) {
    const belly = petalStations[3];
    const p = polar(belly.r, i * 120 + belly.da);
    await cutCircle(p.x, p.y, seedD, hoopH, bodyId, `seed ${i}`);
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
    "Slip-fit trillium. Not a friction washer. Only the inner and outer",
    "diameters are smooth circular lands; the web is air and 2-pass bones.",
    "",
    `Original race ID: ${originalRaceId.toFixed(1)} mm`,
    `Land OD: ${od.toFixed(1)} mm   (assembled FDM slip −${slip.toFixed(2)}; 0.40 printed as a press)`,
    `Funnel OD: ${leadOd.toFixed(1)} mm × ${leadH.toFixed(1)} mm on the bed face`,
    `Bore ID: ${id.toFixed(1)} mm   (clears the Ø${hubOd.toFixed(1)} hub and Ø${journalOd.toFixed(1)} journal — not a fit)`,
    `Bezel wall: ${land.toFixed(1)} mm (4 passes) × ${hoopH.toFixed(1)} mm tall`,
    `Petals: ${petalH.toFixed(1)} mm   tendrils: ${vineH.toFixed(1)} mm   min wall: ${wall2.toFixed(1)} mm (2 passes)`,
    "Material: PLA Basic Orange. Print flat on the X2D, 0.4 mm nozzle.",
    "Suggested: 0.20 mm layer, 2 walls, 15% gyroid, no supports, no brim.",
    "",
    "Print with the funnel on the bed (lead-in DOWN). The bezels stand proud",
    "on top. Drop rollers into the top-load slots, then drop this ring over",
    "the journal funnel-first. It sits on the Y-frame ribs and blocks the",
    "roller inner ends without gripping the race. Rotor on, C-clip on.",
    "",
  ].join("\n");
  writeFileSync(path.join(ringDir, "README.txt"), readme);
  writeFileSync(path.join(ringDir, "Print-Kit-Inner-Ring-design.md"), `# Inner keeper ring\n\n${readme}`);
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
    `  OD ${od.toFixed(1)} / ID ${id.toFixed(1)} / h ${hoopH.toFixed(1)}  body ${ring.bodyId}`,
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
