#!/usr/bin/env node
/**
 * CAD synthesis tutor — rerunnable MCP integration exam.
 *
 * Builds the printed VAWT assembly from
 * scripts/fixtures/print-kit-tutor.spec.json through nbcad-mcp (headless).
 * This is the agent-shaped path: server/discover → tools/call → grade lessons.
 *
 *   npm run test:mcp-print-kit
 *   node scripts/mcp-print-kit-tutor.mjs
 *   node scripts/mcp-print-kit-tutor.mjs --live   # optional UI session
 *
 * Requires native OCCT (OCCT_ROOT on PATH). Does not add modeling tools.
 */
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { mkdirSync, readFileSync, writeFileSync, existsSync, unlinkSync, readdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { strToU8, zipSync } from "fflate";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "..");
const spec = JSON.parse(
  readFileSync(path.join(here, "fixtures", "print-kit-tutor.spec.json"), "utf8"),
);

function defaultBin() {
  if (process.env.NBCAD_MCP_BIN) return process.env.NBCAD_MCP_BIN;
  const release = path.join(repoRoot, "mcp-server", "target", "release", "nbcad-mcp.exe");
  const debug = path.join(repoRoot, "mcp-server", "target", "debug", "nbcad-mcp.exe");
  const releaseUnix = path.join(repoRoot, "mcp-server", "target", "release", "nbcad-mcp");
  const debugUnix = path.join(repoRoot, "mcp-server", "target", "debug", "nbcad-mcp");
  return [release, debug, releaseUnix, debugUnix].find((candidate) => existsSync(candidate));
}

const bin = defaultBin();
const live = process.argv.includes("--live");
const defaultKitDir = path.join(os.homedir(), "Documents", "noBS-CAD");
const out3mfLegacy =
  process.env.NBCAD_3MF_OUT || path.join(defaultKitDir, "Print-Kit-Tutor.3mf");
const out3mfDir =
  process.env.NBCAD_3MF_DIR || out3mfLegacy.replace(/\.3mf$/i, "");
const outProject =
  process.env.NBCAD_PROJECT_OUT || path.join(defaultKitDir, "Print-Kit-Tutor.nbcad");
const outReport =
  process.env.NBCAD_TUTOR_OUT || path.join(defaultKitDir, "Print-Kit-Tutor-report.json");
const outDesign =
  process.env.NBCAD_DESIGN_OUT || path.join(defaultKitDir, "Print-Kit-Tutor-design.md");
const currentPlates = spec.print_plates ?? [
  "01-base",
  "02-axle",
  "03-rotor",
  "04-roller-cartridge",
  "05-retainer",
];
const retiredPlates = spec.retired_print_plates ?? [
  "02-shaft",
  "03-hub",
  "04-wings",
  "05-plate",
  "06-bushing",
  "07-cap",
];

function removeFile(file) {
  if (!existsSync(file)) return false;
  try {
    unlinkSync(file);
    return true;
  } catch {
    return false;
  }
}

/** Drop the assembled nest and any retired/extra plates before writing this run. */
function cleanKitOutputs() {
  const removed = [];
  if (removeFile(out3mfLegacy)) removed.push(path.basename(out3mfLegacy));
  mkdirSync(out3mfDir, { recursive: true });
  const keep = new Set(currentPlates.map((name) => `${name}.3mf`.toLowerCase()));
  for (const name of readdirSync(out3mfDir)) {
    if (keep.has(name.toLowerCase())) continue;
    if (removeFile(path.join(out3mfDir, name))) removed.push(name);
  }
  for (const name of retiredPlates) {
    if (removeFile(path.join(out3mfDir, `${name}.3mf`))) removed.push(`${name}.3mf`);
  }
  return removed;
}

function plateDirListing() {
  if (!existsSync(out3mfDir)) return [];
  return readdirSync(out3mfDir).filter((name) => name.toLowerCase().endsWith(".3mf"));
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
    "io.modelcontextprotocol/clientInfo": { name: "print-kit-tutor", version: "1" },
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

const steps = [];
async function call(name, args = {}, timeoutMs = 120000) {
  const body = toolBody(await request("tools/call", { name, arguments: args }, timeoutMs));
  steps.push({ tool: name, ok: true });
  return body;
}
function requireClean(update, label) {
  const errors = update?.scene?.errors ?? [];
  if (errors.length) throw new Error(`${label}: ${JSON.stringify(errors)}`);
  return update;
}
function lastSketch(doc) {
  return [...(doc.features ?? [])].reverse().find((feature) => feature.kind === "sketch")?.name;
}
function originZ(plane) {
  const origin = plane?.origin;
  if (!origin) return null;
  return Array.isArray(origin) ? origin[2] : origin.z ?? null;
}
function originArr(plane) {
  const origin = plane?.origin;
  if (!origin) return [0, 0, 0];
  return Array.isArray(origin) ? origin : [origin.x ?? 0, origin.y ?? 0, origin.z ?? 0];
}
function sane(n) {
  return Number.isFinite(n) && Math.abs(n) < 1e5;
}
function bboxOf(body) {
  const pts = [];
  for (const face of body?.faces ?? []) if (face.plane) pts.push(originArr(face.plane));
  for (const edge of body?.edges ?? []) {
    for (const point of edge.points ?? []) {
      pts.push(Array.isArray(point) ? point : [point.x, point.y, point.z]);
    }
  }
  const filtered = pts.filter((point) => point.every(sane));
  if (!filtered.length) return null;
  const xs = filtered.map((point) => point[0]);
  const ys = filtered.map((point) => point[1]);
  const zs = filtered.map((point) => point[2]);
  return {
    min: [Math.min(...xs), Math.min(...ys), Math.min(...zs)],
    max: [Math.max(...xs), Math.max(...ys), Math.max(...zs)],
    span: [
      Math.max(...xs) - Math.min(...xs),
      Math.max(...ys) - Math.min(...ys),
      Math.max(...zs) - Math.min(...zs),
    ],
  };
}
function newestBody(update, known) {
  const skip = new Set(known);
  const ids = (update.scene?.bodies ?? []).map((body) => body.id).filter((id) => !skip.has(id));
  if (!ids.length) throw new Error("no new body");
  return Math.max(...ids);
}
function xyz(value) {
  if (!value) return null;
  if (Array.isArray(value)) return [value[0], value[1], value[2]];
  if (value.x == null) return null;
  return [value.x, value.y, value.z];
}

function mm(value) {
  return value * spec.scale;
}
function mmMin(value, min) {
  return Math.max(value * spec.scale, min);
}
function wall() {
  return mmMin(4, spec.nozzle_mm * 4);
}
function teMin() {
  return Math.max(spec.airfoil_te_min_mm, spec.nozzle_mm * 2);
}
function rollerD() {
  return mmMin(spec.roller_d, spec.roller_min_d);
}
function rollerH() {
  return mmMin(spec.roller_h, 6);
}
function innerRaceD() {
  return mmMin(spec.inner_race_d, rollerD() + 4);
}
function hubBore() {
  return innerRaceD() + 2 * rollerD() + spec.fit_running_mm;
}
function hubOd() {
  return hubBore() + 2 * wall() * 2;
}
function hubH() {
  return Math.max(mmMin(spec.hub_h, 8), rollerH());
}
function hubSquare() {
  return mm(spec.axle_square) + spec.fit_friction_mm;
}
function axleSquare() {
  return mm(spec.axle_square);
}
function axleFlangeD() {
  return Math.max(mm(spec.axle_flange_d), hubOd() + 6);
}
function axleFlangeH() {
  return mmMin(spec.axle_flange_h, 2.4);
}
function wingH() {
  return mm(spec.wing_h);
}
function wingRadius() {
  return mm(spec.wing_radius);
}
function chordRoot() {
  return mm(spec.wing_chord_root);
}
function chordTip() {
  return mm(spec.wing_chord_tip);
}
function wingThick() {
  return chordRoot() * spec.airfoil_t_c;
}
function baseH() {
  return mmMin(spec.base_h, 6);
}
function ribW() {
  return mmMin(spec.rib_w, 5);
}
function padD() {
  return mmMin(spec.pad_d, 10);
}
function postCircleR() {
  return mm(spec.post_circle_r);
}
function baseBossD() {
  return Math.max(axleFlangeD(), hubOd() + 8, mm(spec.base_boss_d));
}
function cageOd() {
  return hubBore() - spec.fit_running_mm;
}
function cageId() {
  return innerRaceD() + spec.fit_slip_mm;
}
function cageH() {
  return mmMin(spec.cage_h, rollerH() + 2);
}
function cagePocket() {
  return rollerD() + spec.fit_running_mm;
}
function retainerOd() {
  return Math.max(Math.min(axleFlangeD(), hubOd() + 4), hubBore() + 4);
}
function retainerId() {
  return axleSquare() + spec.fit_slip_mm;
}
function retainerH() {
  return mmMin(spec.retainer_h, 2);
}
function pcd() {
  return (innerRaceD() + hubBore()) * 0.5;
}
function usableBed() {
  return spec.printer.bed_mm.map((n) => n - 2 * spec.printer.margin_mm);
}
function bladeTipR() {
  return wingRadius() + chordTip() * 0.15;
}
function rotorD() {
  return bladeTipR() * 2;
}
function baseEnvelope() {
  return postCircleR() * 2 + padD();
}
function rotorPrintH() {
  return hubH() + wingH();
}
function flangeZ() {
  return baseH() + spec.thrust_float;
}
function raceZ() {
  return flangeZ() + axleFlangeH();
}
function hubZ() {
  return raceZ();
}
function retainerZ() {
  return hubZ() + hubH();
}
function postH() {
  return retainerZ() + retainerH() + spec.thrust_float;
}
function wingAngleDeg(index) {
  return spec.wing_offset_deg + (360 / Math.max(spec.wing_count, 1)) * index;
}
function helixAzimuthDeg(index, t) {
  return wingAngleDeg(index) + spec.helix_deg * t;
}
function helixCenter(index, t) {
  const radians = (helixAzimuthDeg(index, t) * Math.PI) / 180;
  return [wingRadius() * Math.cos(radians), wingRadius() * Math.sin(radians)];
}
function postXY(index) {
  const radians = ((360 / Math.max(spec.post_count, 1)) * index * Math.PI) / 180;
  return [postCircleR() * Math.cos(radians), postCircleR() * Math.sin(radians)];
}
function rollerXY(index) {
  const radians = ((360 / Math.max(spec.roller_count, 1)) * index * Math.PI) / 180;
  const r = pcd() * 0.5;
  return [r * Math.cos(radians), r * Math.sin(radians)];
}
function solidity() {
  return (spec.wing_count * chordRoot()) / (Math.PI * rotorD());
}
function airfoilOk() {
  return (
    /NACA/i.test(spec.airfoil) &&
    spec.airfoil_t_c >= 0.18 &&
    spec.airfoil_t_c <= 0.26 &&
    Math.abs(spec.wing_thick - spec.wing_chord_root * spec.airfoil_t_c) < 0.2 &&
    teMin() + 1e-9 >= spec.nozzle_mm * 2 &&
    chordRoot() > chordTip()
  );
}
function fitsOk() {
  return (
    Math.abs(spec.fit_running_mm - spec.nozzle_mm) < 1e-9 &&
    spec.fit_friction_mm + 1e-9 < spec.fit_slip_mm &&
    spec.fit_slip_mm + 1e-9 < spec.fit_running_mm &&
    Math.abs(spec.clearance_mm - spec.fit_running_mm) < 1e-9
  );
}
function rollersOk() {
  return (
    spec.roller_count >= 6 &&
    rollerD() + 1e-9 >= spec.roller_min_d &&
    pcd() > innerRaceD() &&
    hubBore() > innerRaceD() + 2 * rollerD() &&
    cageOd() + 1e-9 < hubBore() &&
    axleFlangeD() + 1e-9 > hubOd()
  );
}
function helixOk() {
  return spec.helix_deg >= 45 && spec.helix_stations >= 2;
}
function fitsX2dAtMax() {
  const bed = usableBed();
  return (
    rotorD() / spec.scale <= bed[0] &&
    rotorPrintH() / spec.scale <= bed[2] &&
    baseEnvelope() / spec.scale <= bed[0]
  );
}
function scaleOk() {
  return spec.scale > 0 && spec.scale <= spec.max_scale + 1e-9 && fitsX2dAtMax() && spec.printer.bed_mm[2] >= 260;
}
function printFlatOk() {
  const axleH = axleFlangeH() + cageH();
  return axleH <= axleFlangeD() && rotorPrintH() > axleH * 3;
}
function sanityOk() {
  return (
    baseEnvelope() / rotorD() <= 1.55 &&
    wingH() + 1e-9 >= chordRoot() * 2.5 &&
    solidity() >= 0.24 &&
    solidity() <= 0.45
  );
}
function estimatedSolidCm3() {
  const hub = Math.PI * ((hubOd() * 0.5) ** 2 - (hubBore() * 0.5) ** 2) * hubH();
  const wings =
    spec.wing_count * 0.62 * ((chordRoot() + chordTip()) * 0.5) * wingThick() * wingH();
  const base =
    Math.PI * (baseBossD() * 0.5) ** 2 * baseH() + spec.post_count * ribW() * postCircleR() * baseH();
  const axle =
    Math.PI * (axleFlangeD() * 0.5) ** 2 * axleFlangeH() +
    Math.PI * (innerRaceD() * 0.5) ** 2 * cageH();
  const cage = Math.PI * ((cageOd() * 0.5) ** 2 - (cageId() * 0.5) ** 2) * cageH();
  const rollers = spec.roller_count * Math.PI * (rollerD() * 0.5) ** 2 * rollerH();
  const retainer = Math.PI * ((retainerOd() * 0.5) ** 2 - (retainerId() * 0.5) ** 2) * retainerH();
  return (hub + wings + base + axle + cage + rollers + retainer) / 1000;
}
function estimatedPrintMassG() {
  return estimatedSolidCm3() * spec.filament.density_g_cm3 * spec.filament.print_volume_factor;
}
function estimatedFilamentUsd() {
  return (estimatedPrintMassG() / 1000) * spec.filament.price_usd_per_kg;
}

async function beginXY() {
  await call("cad_set_focus", { focus: "sketch", explicit: true });
  await call("sketch_begin", { plane: { type: "origin_plane", plane: "xy" } });
  await call("sketch_set_grid_snap", { enabled: false });
}
async function beginDatum(datumId) {
  await call("cad_set_focus", { focus: "sketch", explicit: true });
  await call("sketch_begin", { plane: { type: "datum_plane", datum_id: datumId } });
  await call("sketch_set_grid_snap", { enabled: false });
}
async function finishSketch() {
  await call("sketch_finish");
  return lastSketch(await call("cad_document"));
}
async function offsetXY(z) {
  await call("cad_set_focus", { focus: "datums", explicit: true });
  const created = await call("construction_plane_offset", {
    reference: { type: "origin_plane", plane: "xy" },
    distance: z,
  });
  const planes = created.planes ?? [];
  const hit =
    [...planes].reverse().find((plane) => Math.abs((originZ(plane.basis) ?? 999) - z) < 0.25) ??
    planes[planes.length - 1];
  if (!hit?.datum_id) throw new Error(`no datum at z=${z}`);
  return hit.datum_id;
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
function naca00Thickness(x, thicknessRatio) {
  const t = Math.min(1, Math.max(0, x));
  return (
    5 *
    thicknessRatio *
    (0.2969 * Math.sqrt(t) - 0.126 * t - 0.3516 * t * t + 0.2843 * t ** 3 - 0.1015 * t ** 4)
  );
}
function nacaSymmetricLoop(chord, thicknessRatio, stations, te) {
  const count = Math.max(stations, 6);
  const xs = [];
  for (let i = 0; i < count; i++) {
    const beta = (Math.PI * i) / (count - 1);
    xs.push(0.5 * (1 - Math.cos(beta)));
  }
  const upper = [];
  const lower = [];
  for (const x of xs) {
    let yt = naca00Thickness(x, thicknessRatio) * chord;
    if (x > 0.85) yt = Math.max(yt, te / 2);
    const xc = (x - 0.5) * chord;
    upper.push({ x: xc, y: yt });
    lower.push({ x: xc, y: -yt });
  }
  const points = [...upper];
  for (let i = lower.length - 2; i >= 0; i--) points.push(lower[i]);
  if (points[0]) points.push(points[0]);
  return points;
}
async function addAirfoil(center, angleDeg, chord, thicknessRatio, stations, te) {
  const angle = (angleDeg * Math.PI) / 180;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const world = nacaSymmetricLoop(chord, thicknessRatio, stations, te).map((p) => ({
    x: center[0] + cos * p.x - sin * p.y,
    y: center[1] + sin * p.x + cos * p.y,
  }));
  await addPoly(world, true);
}

async function buildBase() {
  await beginXY();
  await addCircle(0, 0, baseBossD());
  let sketch = await finishSketch();
  let update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: baseH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "base boss",
  );
  const baseId = newestBody(update, []);
  for (let i = 0; i < spec.post_count; i++) {
    const [px, py] = postXY(i);
    const angle = (360 / spec.post_count) * i;
    await beginXY();
    await addOrientedRect([px * 0.5, py * 0.5], postCircleR(), ribW(), angle);
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "join",
        extent: { type: "distance", distance: baseH() },
        taper_angle_deg: 0,
        flip: false,
        target_body_ids: [baseId],
      }),
      `base rib ${i}`,
    );
    await beginXY();
    await addCircle(px, py, padD());
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "join",
        extent: { type: "distance", distance: baseH() },
        taper_angle_deg: 0,
        flip: false,
        target_body_ids: [baseId],
      }),
      `base pad ${i}`,
    );
  }
  await beginXY();
  await addOrientedRect([0, 0], axleSquare(), axleSquare(), 0);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "join",
      extent: { type: "distance", distance: postH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [baseId],
    }),
    "base square post",
  );
  return baseId;
}

async function buildAxle(known) {
  const deck = await offsetXY(flangeZ());
  await beginDatum(deck);
  await addCircle(0, 0, axleFlangeD());
  let sketch = await finishSketch();
  let update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: axleFlangeH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "axle flange",
  );
  const axleId = newestBody(update, known);
  await beginDatum(await offsetXY(raceZ()));
  await addCircle(0, 0, innerRaceD());
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "join",
      extent: { type: "distance", distance: cageH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [axleId],
    }),
    "axle inner race",
  );
  await beginDatum(deck);
  await addOrientedRect([0, 0], hubSquare(), hubSquare(), 0);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: axleFlangeH() + cageH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [axleId],
    }),
    "axle square bore",
  );
  return axleId;
}

async function buildRotor(known) {
  const z0 = hubZ();
  const deck = await offsetXY(z0);
  await beginDatum(deck);
  await addCircle(0, 0, hubOd());
  await addCircle(0, 0, hubBore());
  let sketch = await finishSketch();
  let update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: hubH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "rotor hub",
  );
  const rotorId = newestBody(update, known);
  for (let index = 0; index < spec.wing_count; index++) {
    const [px, py] = helixCenter(index, 0);
    await beginDatum(await offsetXY(z0));
    await addOrientedRect(
      [px * 0.5, py * 0.5],
      wingRadius() + chordRoot() * 0.5,
      wall() * 3,
      wingAngleDeg(index),
    );
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "join",
        extent: { type: "distance", distance: hubH() },
        taper_angle_deg: 0,
        flip: false,
        target_body_ids: [rotorId],
      }),
      `blade spar ${index}`,
    );
    const stations = Math.max(spec.helix_stations, 2);
    const sections = [];
    for (let station = 0; station < stations; station++) {
      const t = station / (stations - 1);
      const chord = chordRoot() * (1 - t) + chordTip() * t;
      await beginDatum(await offsetXY(z0 + hubH() * 0.35 + wingH() * t));
      await addAirfoil(
        helixCenter(index, t),
        helixAzimuthDeg(index, t) + 90,
        chord,
        spec.airfoil_t_c,
        spec.airfoil_stations,
        teMin(),
      );
      sections.push({ sketch_name: await finishSketch(), profile_index: 0 });
    }
    requireClean(
      await call("solid_loft", {
        sections,
        ruled: false,
        operation: "join",
        target_body_ids: [rotorId],
        continuity: "g1",
      }),
      `helical blade ${index}`,
    );
  }
  return rotorId;
}

async function buildCartridge(known) {
  const deck = await offsetXY(raceZ());
  await beginDatum(deck);
  await addCircle(0, 0, cageOd());
  await addCircle(0, 0, cageId());
  let sketch = await finishSketch();
  let update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: cageH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "roller cage",
  );
  const cageIdBody = newestBody(update, known);
  for (let i = 0; i < spec.roller_count; i++) {
    const [x, y] = rollerXY(i);
    await beginDatum(deck);
    await addCircle(x, y, cagePocket());
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "cut",
        extent: { type: "distance", distance: cageH() },
        taper_angle_deg: 0,
        flip: false,
        target_body_ids: [cageIdBody],
      }),
      `cage pocket ${i}`,
    );
  }
  const rollerIds = [];
  const seen = [...known, cageIdBody];
  for (let i = 0; i < spec.roller_count; i++) {
    const [x, y] = rollerXY(i);
    await beginDatum(deck);
    await addCircle(x, y, rollerD());
    sketch = await finishSketch();
    update = requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "new_body",
        extent: { type: "distance", distance: rollerH() },
        taper_angle_deg: 0,
        flip: false,
        target_body_ids: [],
      }),
      `roller ${i}`,
    );
    const id = newestBody(update, seen);
    seen.push(id);
    rollerIds.push(id);
  }
  return { cageId: cageIdBody, rollerIds };
}

async function buildRetainer(known) {
  await beginDatum(await offsetXY(retainerZ()));
  await addCircle(0, 0, retainerOd());
  await addCircle(0, 0, retainerId());
  const sketch = await finishSketch();
  const update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: retainerH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "retainer",
  );
  return newestBody(update, known);
}

function planarFaceNearZ(scene, bodyId, z) {
  const body = (scene.bodies ?? []).find((item) => item.id === bodyId);
  if (!body) return null;
  let best = null;
  for (const face of body.faces ?? []) {
    const plane = face.plane;
    if (!plane) continue;
    const origin = xyz(plane.origin);
    const normal = xyz(plane.normal);
    if (!origin || !normal || Math.abs(normal[2]) < 0.85) continue;
    const score = Math.abs(origin[2] - z);
    if (!best || score < best.score) best = { face, plane, score };
  }
  if (!best) return null;
  return {
    body_id: bodyId,
    face_id: best.face.id,
    face_key: best.face.key,
    kind: "planar_face",
    frame: {
      origin: best.plane.origin,
      primary_axis: best.plane.normal,
      secondary_axis: best.plane.u,
    },
  };
}

async function formAssembly(ids) {
  await call("cad_set_focus", { focus: "assembly", explicit: true });
  const components = [];
  for (const [name, bodyIds] of [
    ["base", [ids.baseId]],
    ["axle", [ids.axleId]],
    ["rotor", [ids.rotorId]],
    ["roller_cage", [ids.cageId, ...ids.rollerIds]],
    ["retainer", [ids.retainerId]],
  ]) {
    const created = await call("assembly_create_component", {
      name,
      body_ids: bodyIds,
      absorb_promoted_bodies: true,
    });
    const componentId = created.id ?? created.component?.id;
    if (!componentId) throw new Error(`component ${name} missing id: ${JSON.stringify(created)}`);
    const occurrence = await call("assembly_create_occurrence", {
      component_id: componentId,
      name: `${name}_1`,
    });
    components.push({ name, componentId, occurrence });
  }
  const grounded = components[0];
  if (grounded?.occurrence?.id) {
    await call("assembly_set_occurrence_grounded", {
      occurrence_id: grounded.occurrence.id,
      grounded: true,
    });
  }
  try {
    await call("assembly_set_grounded_body", { body_id: ids.baseId });
  } catch {
    /* optional on some hosts */
  }
  const scene = await call("solid_scene");
  const axleFace = planarFaceNearZ(scene, ids.axleId, raceZ() + cageH());
  const rotorFace = planarFaceNearZ(scene, ids.rotorId, hubZ() + hubH());
  if (axleFace && rotorFace) {
    await call("assembly_create_joint", {
      name: "rotor_spin",
      kind: "revolute",
      connector_a: axleFace,
      connector_b: rotorFace,
      grounded_body_id: ids.axleId,
    });
  }
  const document = await call("assembly_document");
  const defs = document.component_structure?.definitions?.length ?? 0;
  if (defs < 5) throw new Error(`expected 5 components, got ${defs}`);
  return document;
}

async function makeAssemblyDrawing() {
  await call("cad_set_focus", { focus: "drawing", explicit: true });
  await call("cad_drawing_create_sheet", {
    standard: "iso",
    format: "a3",
    orientation: "landscape",
    title: spec.title,
    drawing_number: "PK-VAWT-001",
    revision: "A",
  });
  await call("cad_drawing_auto_layout", {});
  const notes = [
    [
      [18, 28],
      `ASSEMBLY  scale=${spec.scale.toFixed(2)} (1.0 = ${spec.printer.name} max)  PLA  nozzle ${spec.nozzle_mm} mm`,
    ],
    [
      [18, 38],
      `FITS  running +${spec.fit_running_mm.toFixed(2)}  slip +${spec.fit_slip_mm.toFixed(2)}  friction +${spec.fit_friction_mm.toFixed(2)}  (slicer XY hole comp = 0)`,
    ],
    [
      [18, 48],
      "PRINT  base/axle/cage/retainer FLAT. Rotor STANDING on hub, open drafted tips. Cartridge is PIP.",
    ],
    [
      [18, 58],
      `BOM  base (Y-frame + square post) · axle (inner-race puck) · rotor (hub+3×${spec.airfoil}) · roller cage + ${spec.roller_count} PIP rollers · retainer`,
    ],
    [
      [18, 68],
      `ROLLERS  Ø${rollerD().toFixed(1)} × ${rollerH().toFixed(1)}  PCD ${pcd().toFixed(1)}  for blade-tip moment. No metal 608.`,
    ],
  ];
  for (const [position, text] of notes) {
    await call("cad_drawing_add_note", { position, text });
  }
  const drawing = await call("cad_drawing_document");
  if ((drawing.sheets?.length ?? 0) < 1) throw new Error("drawing sheet missing");
}

function writeDesignReport({ bodies, rotorBox, rotorFaces, plateFiles }) {
  const iterations = [
    ["Print-bed scatter", "Parts did not assemble on one axis. Not a machine."],
    ["Colliding spinner", "Rotor swept the posts. Could not rotate."],
    ["Helical C-buckets", "Leftover loft, not a mount, not an airfoil."],
    ["Hoop sector r20–r28", "Concentric C. Concave faces the axis. No net torque."],
    ["Turntable / lazy Susan", "Competent bearings, no wing. The frame had nothing to do."],
    ["Flat plate 12×2.4×32", "A vane, not a 2026 symmetric section. Directionless VAWT needs an airfoil."],
    ["Straight NACA in a Ø90 cage", "Section was right; girth was wrong. Prismatic blades idle most of the rev."],
    ["Uniform +0.40 on every hole", "Easy parts stayed easy; tight locates were sloppy. Role-based running/slip/friction."],
    ["Tenoned separate wings", "Three blades plus a hub is one printed rotor, not a puzzle."],
    ["Tall skinny shaft + two-land sleeve", "Cannot take blade-tip moment and needs a support tower. Short post + large-PCD roller pack."],
  ];
  const usd = estimatedFilamentUsd().toFixed(2);
  const markdown = `# Print Kit Tutor — design report

Spec \`${spec.id}\` · ${spec.title} · nozzle ${spec.nozzle_mm} mm · scale ${spec.scale} of ${spec.printer.name}

## 1. Iteration log

| Iteration | Why it failed / what changed |
|-----------|------------------------------|
${iterations.map(([name, why]) => `| ${name} | ${why} |`).join("\n")}

## 2. Design process

- **Architecture:** Helical H-Darrieus, directionless (no yaw). Short fixed square post. Hub freewheels on a printed roller pack. No tall mast.
- **Airfoil:** ${spec.airfoil} (t/c ${spec.airfoil_t_c}). 2026 VAWT dynamic-stall work favors t/c 21–24%. TE blunt to ${teMin()} mm (≥ 2 nozzles). Open drafted tips.
- **Rotor:** one piece, N=${spec.wing_count}, c=${chordRoot().toFixed(1)}/${chordTip().toFixed(1)} mm, R=${wingRadius().toFixed(1)} mm, span=${wingH().toFixed(1)} mm, helix ${spec.helix_deg}°, σ=${solidity().toFixed(3)}. Envelope/rotor ${(baseEnvelope() / rotorD()).toFixed(2)}.
- **Fits:** running +${spec.fit_running_mm} (rollers), slip +${spec.fit_slip_mm} (retainer), friction +${spec.fit_friction_mm} (axle on post). Slicer XY hole compensation stays 0. Thrust land float ${spec.thrust_float}. No press. No metal 608.
- **Bushing:** printed roller cartridge ${spec.roller_count}× Ø${rollerD().toFixed(1)} on PCD ${pcd().toFixed(1)}. A two-land sleeve is not enough for tip moment.
- **Scale:** source numbers are X2D-max (256×256×260, 8 mm margin). Exam scale ${spec.scale}. Feature floors: roller Ø${spec.roller_min_d}, TE ${spec.airfoil_te_min_mm}, 4-nozzle walls.
- **Service finish:** rotor standing so layer lines run spanwise; sand PLA 400→1000 on skins. Do not vapor-smooth a running fit.
- **Assembly drawing:** A3 sheet, auto-layout, notes for fits / scale / print / BOM.

## 3. Final product

Five functional parts, assembly order: ${spec.assembly_order.join(" → ")}.

| Part | Count | Role |
|------|------:|------|
| Base | 1 | Y-frame + square stator post. Print flat. |
| Axle | 1 | Inner-race puck, square bore, print on the flange. |
| Rotor | 1 | Hub + 3× ${spec.airfoil}, open tips, print standing. |
| Roller cartridge | 1 | Cage + ${spec.roller_count} PIP rollers. |
| Retainer | 1 | Slip ring on the post. |

Rotor bbox (exam): ${rotorBox ? `${rotorBox.span.map((n) => n.toFixed(1)).join(" × ")} mm` : "n/a"}; faces=${rotorFaces}. Bodies=${bodies.length}.

## 4. Printing cost (plastic / material)

Assumptions: ${spec.filament.name}, ${spec.filament.density_g_cm3} g/cm³, $${spec.filament.price_usd_per_kg}/kg, print-volume factor ${spec.filament.print_volume_factor}.

| | Value |
|--|------:|
| CAD solid (estimate) | ${estimatedSolidCm3().toFixed(1)} cm³ |
| Estimated print mass | ${estimatedPrintMassG().toFixed(1)} g |
| Filament cost | **$${usd}** |

Print plates in \`${out3mfDir}\` (folder wiped first; cartridge is PIP; do not print the assembled nest):

${plateFiles.map((file) => `- \`${file}\``).join("\n")}

Slicer: drop each plate to the bed. Base/axle/cage/retainer flat. Rotor standing on the hub, tips up.

Project: \`${outProject}\`

Electricity and machine time are not priced. No additional hardware.
`;
  return {
    ok:
      markdown.includes("Iteration log") &&
      estimatedFilamentUsd() > 0.05 &&
      /NACA/i.test(spec.airfoil) &&
      plateFiles.length >= spec.min_print_plates,
    markdown,
  };
}

function record(lessons, id, pass, detail) {
  lessons.push({ id, pass: !!pass, detail });
}

const report = { ok: false, spec: spec.id, lessons: [], steps };
try {
  await request("server/discover", {});
  const prompt = toolBody(await request("prompts/get", { name: "model_print_kit", arguments: {} }));
  const recipe = prompt?.messages?.[0]?.content?.text ?? "";
  if (!/thrust|cone|clearance|nozzle/i.test(recipe)) {
    throw new Error("model_print_kit prompt is missing the FDM curriculum");
  }
  if (
    !/airfoil|NACA 0021|directionless|bushing|design report|service finish|helical|girth|two-land|Y-frame|print plate/i.test(
      recipe,
    )
  ) {
    throw new Error("model_print_kit prompt is missing the 2026 VAWT design contract");
  }

  if (live) {
    const sessions = await call("cad_list_sessions");
    const want = process.env.NBCAD_SESSION_ID;
    const chosen =
      (sessions.session_details ?? []).find((session) => session.session_id === want) ??
      (sessions.session_details ?? []).find((session) => session.has_model && session.heartbeat?.stale === false);
    if (!chosen) throw new Error("--live requested but no attachable session");
    await call("cad_attach", { session_id: chosen.session_id, mode: "live" });
  }

  await call("cad_new_project");
  await call("cad_set_document_name", { name: spec.document_name });

  const baseId = await buildBase();
  const axleId = await buildAxle([baseId]);
  const rotorId = await buildRotor([baseId, axleId]);
  const { cageId, rollerIds } = await buildCartridge([baseId, axleId, rotorId]);
  const retainerId = await buildRetainer([baseId, axleId, rotorId, cageId, ...rollerIds]);

  let assemblyOk = false;
  let assemblyDetail = "";
  try {
    await formAssembly({ baseId, axleId, rotorId, cageId, rollerIds, retainerId });
    assemblyOk = true;
    assemblyDetail = "5 components; hub freewheels on the fixed post";
  } catch (error) {
    assemblyDetail = String(error?.message ?? error);
  }

  let drawingOk = false;
  let drawingDetail = "";
  try {
    await makeAssemblyDrawing();
    drawingOk = true;
    drawingDetail = "A3 assembly sheet with fit / scale / print / BOM notes";
  } catch (error) {
    drawingDetail = String(error?.message ?? error);
  }

  await call("cad_set_focus", { focus: "print", explicit: true });
  const appearances = [
    [baseId, "bambu.pla.basic.black"],
    [axleId, "bambu.pla.basic.jade_white"],
    [rotorId, "bambu.pla.basic.green"],
    [cageId, "bambu.pla.matte.dark_gray"],
    [retainerId, "bambu.pla.basic.red"],
  ];
  for (const [id, preset] of appearances) {
    await call("set_body_appearance", { body_id: id, preset_id: preset });
  }
  for (const id of rollerIds) {
    await call("set_body_appearance", { body_id: id, preset_id: "bambu.pla.basic.orange" });
  }
  const preflight = await call("solid_export_preflight");
  const removedPlates = cleanKitOutputs();
  const plateBodies = {
    "01-base": [baseId],
    "02-axle": [axleId],
    "03-rotor": [rotorId],
    "04-roller-cartridge": [cageId, ...rollerIds],
    "05-retainer": [retainerId],
  };
  const plateFiles = [];
  const plateBytes = [];
  for (const name of currentPlates) {
    const bodyIds = plateBodies[name];
    if (!bodyIds) throw new Error(`no bodies mapped for plate ${name}`);
    const exported = await call("solid_export_3mf", {
      slicer_target: spec.slicer_target,
      include_appearance: true,
      body_ids: bodyIds,
    });
    const bytes = Buffer.from(exported.bytes_base64, "base64");
    const dest = path.join(out3mfDir, `${name}.3mf`);
    writeFileSync(dest, bytes);
    plateFiles.push(dest);
    plateBytes.push(bytes);
  }
  const leftoverPlates = plateDirListing().filter(
    (name) => !currentPlates.includes(name.replace(/\.3mf$/i, "")),
  );
  const scene = await call("solid_scene");
  const document = await call("cad_document");
  const assembly = await call("assembly_document").catch(() => ({}));
  const project = await call("cad_project_model");
  const modelJson =
    typeof project === "string"
      ? project
      : typeof project?.model_json === "string"
        ? project.model_json
        : JSON.stringify(project);
  mkdirSync(path.dirname(outProject), { recursive: true });
  const projectBytes = writeNbcadArchive(modelJson, outProject);
  if (live) {
    try {
      await call("cad_detach");
    } catch {
      /* ignore */
    }
  }

  const bodies = scene.bodies ?? [];
  const features = document.features ?? [];
  const rotor = bodies.find((body) => body.id === rotorId);
  const rotorBox = bboxOf(rotor);
  const rotorFaces = rotor?.faces?.length ?? 0;
  const rotorSpan = rotorBox?.span[2] ?? 0;
  const componentCount = assembly.component_structure?.definitions?.length ?? 0;
  const jointCount = assembly.joints?.length ?? 0;

  record(
    report.lessons,
    "fits",
    fitsOk(),
    `running +${spec.fit_running_mm.toFixed(2)}  slip +${spec.fit_slip_mm.toFixed(2)}  friction +${spec.fit_friction_mm.toFixed(2)}`,
  );
  record(
    report.lessons,
    "no_press",
    spec.fit_friction_mm > 0 && spec.fit_friction_mm < spec.nozzle_mm,
    "friction locate is a modeled gap; retain with flange/retainer",
  );
  record(
    report.lessons,
    "assemble",
    assemblyOk && componentCount >= 5 && bodies.length >= spec.min_bodies && rollerIds.length === spec.roller_count,
    `${bodies.length} bodies, ${componentCount} components, ${jointCount} joints; ${assemblyDetail}`,
  );
  record(
    report.lessons,
    "rollers",
    rollersOk() && rollerIds.length === spec.roller_count,
    `${spec.roller_count}× Ø${rollerD().toFixed(1)} rollers on PCD ${pcd().toFixed(1)}; hub bore ${hubBore().toFixed(1)}`,
  );
  record(
    report.lessons,
    "even",
    spec.wing_count === 3 &&
      spec.post_count === 3 &&
      Math.abs(spec.wing_offset_deg + spec.helix_deg * 0.5 - 60) < 1e-9,
    "3 blades at 120°, 60° helix from 30° root",
  );
  record(
    report.lessons,
    "one_piece_rotor",
    rotorFaces >= spec.min_rotor_faces && rotorSpan > wingH() * 0.7 && chordRoot() > chordTip(),
    `rotor faces=${rotorFaces} span=${rotorSpan.toFixed(1)} (hub+3 blades, open tips)`,
  );
  record(
    report.lessons,
    "airfoil",
    airfoilOk() && rotorFaces >= spec.min_rotor_faces,
    `${spec.airfoil} t/c=${spec.airfoil_t_c} TE≥${teMin().toFixed(1)}; σ=${solidity().toFixed(2)}; root/tip chord ${chordRoot().toFixed(1)}/${chordTip().toFixed(1)}`,
  );
  record(
    report.lessons,
    "scale",
    scaleOk() && sanityOk(),
    `scale ${spec.scale.toFixed(2)} of ${spec.printer.name}  rotor Ø${rotorD().toFixed(0)} h${rotorPrintH().toFixed(0)}  bed ${spec.printer.bed_mm.join("×")}`,
  );
  record(
    report.lessons,
    "print_flat",
    printFlatOk(),
    `axle puck h${(axleFlangeH() + cageH()).toFixed(1)} on flange Ø${axleFlangeD().toFixed(1)}; rotor stands ${rotorPrintH().toFixed(0)}`,
  );
  record(
    report.lessons,
    "helix",
    helixOk() && rotorSpan > wingH() * 0.7,
    `${spec.helix_deg}° helix, ${spec.helix_stations} stations, open tips`,
  );
  record(report.lessons, "drawing", drawingOk, drawingDetail);
  const design = writeDesignReport({
    bodies,
    rotorBox,
    rotorFaces,
    plateFiles,
  });
  writeFileSync(outDesign, design.markdown);
  record(
    report.lessons,
    "report",
    design.ok && estimatedFilamentUsd() > 0.05 && airfoilOk(),
    {
      path: outDesign,
      solid_cm3: +estimatedSolidCm3().toFixed(2),
      print_mass_g: +estimatedPrintMassG().toFixed(1),
      filament_usd: +estimatedFilamentUsd().toFixed(2),
      filament: spec.filament.name,
    },
  );
  const platesOk =
    plateBytes.length >= spec.min_print_plates &&
    plateBytes.every((bytes) => bytes[0] === 0x50 && bytes[1] === 0x4b && bytes.length > 32);
  const dirClean =
    leftoverPlates.length === 0 &&
    !existsSync(out3mfLegacy) &&
    plateDirListing().length === currentPlates.length;
  record(
    report.lessons,
    "export",
    features.every((feature) => feature.status?.state !== "error") &&
      (preflight.ok === true || (preflight.timeline_errors ?? []).length === 0) &&
      platesOk &&
      dirClean,
    {
      plates: plateFiles.length,
      bytes: plateBytes.reduce((sum, bytes) => sum + bytes.length, 0),
      preflight: preflight.ok,
      dir: out3mfDir,
      removed: removedPlates,
      leftover: leftoverPlates,
    },
  );
  report.bodies = bodies.map((body) => ({
    id: body.id,
    faces: body.faces?.length,
    bbox: bboxOf(body),
  }));
  report.export = {
    dir: out3mfDir,
    plates: plateFiles,
    byte_length: plateBytes.reduce((sum, bytes) => sum + bytes.length, 0),
    project: outProject,
    project_bytes: projectBytes,
  };
  report.ok = report.lessons.every((lesson) => lesson.pass);
} catch (error) {
  report.error = String(error?.stack ?? error);
  if (live) {
    try {
      await call("cad_detach");
    } catch {
      /* ignore */
    }
  }
} finally {
  mkdirSync(path.dirname(outReport), { recursive: true });
  writeFileSync(outReport, JSON.stringify(report, null, 2));
  console.log(`\nCAD synthesis tutor — ${spec.title}`);
  console.log(
    `Spec ${spec.id}  nozzle ${spec.nozzle_mm} mm  scale ${spec.scale}  running +${spec.fit_running_mm} / slip +${spec.fit_slip_mm} / friction +${spec.fit_friction_mm}`,
  );
  for (const lesson of spec.lessons) {
    const result = report.lessons.find((item) => item.id === lesson.id);
    const mark = result?.pass ? "PASS" : "FAIL";
    console.log(`\n[${mark}] ${lesson.title}`);
    console.log(`  ${lesson.teach}`);
    if (result?.detail) {
      console.log(`  ${typeof result.detail === "string" ? result.detail : JSON.stringify(result.detail)}`);
    }
  }
  if (report.error) console.log(`\nERROR ${report.error}`);
  console.log(`\n${report.ok ? "READY TO PRINT" : "NOT READY"}  report ${outReport}`);
  try {
    child.stdin.end();
  } catch {
    /* ignore */
  }
  setTimeout(() => {
    if (!child.killed) child.kill();
    process.exit(report.ok ? 0 : 1);
  }, 800);
}
