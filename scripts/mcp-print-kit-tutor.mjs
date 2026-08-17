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
const currentPlates = spec.print_plates ?? ["01-kit"];
const retiredPlates = spec.retired_print_plates ?? [
  "01-base",
  "02-axle",
  "02-shaft",
  "03-hub",
  "03-rotor",
  "04-roller-cartridge",
  "04-wings",
  "05-plate",
  "05-retainer",
  "06-bushing",
  "07-cap",
];
const plaOrange = spec.materials?.orange ?? "bambu.pla.basic.orange";
const plaGlow = spec.materials?.glow ?? "bambu.pla.glow.green";

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
function originX(plane) {
  const origin = plane?.origin;
  if (!origin) return null;
  return Array.isArray(origin) ? origin[0] : origin.x ?? null;
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
function packH() {
  // Pack height is the roller diameter: lying cylinders, axis radial.
  return rollerD();
}
function rollerLen() {
  return mmMin(spec.roller_len, 8);
}
function pocketLen() {
  return rollerLen() + spec.fit_running_mm;
}
function innerRaceD() {
  return mmMin(spec.inner_race_d, 12);
}
function plateBore() {
  return innerRaceD() + spec.fit_running_mm;
}
function hubOd() {
  return hubDeckOd();
}
function hubH() {
  return hubDeckH();
}
function hubDeckH() {
  return Math.max(mmMin(10, 5), bedReliefH());
}
function hubDeckOd() {
  return (wingRadius() + chordRoot() * 0.18) * 2;
}
function bladeLoftZ() {
  return hubZ() + hubDeckH();
}
function plateZ() {
  return cageZ() + packH() + spec.thrust_float;
}
function bladeRootZ() {
  return hubZ() + hubDeckH();
}
function hubSquare() {
  return mm(spec.axle_square) + spec.fit_friction_mm;
}
function axleSquare() {
  return mm(spec.axle_square);
}
function axleFlangeD() {
  // Race covers the rollers and the cage rim. Stay inside the plate —
  // cageOd()+4 was the orange halo under a smaller deck.
  const race = pcd() + rollerLen() + 2;
  return Math.min(Math.max(race, cageOd()), hubDeckOd() - 2);
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
  // The Y-frame center *is* the lower race. One printed stator.
  return axleFlangeD();
}
function topLoad() {
  return spec.nozzle_mm * 2;
}
function topLoadPocket() {
  return rollerD() + spec.fit_running_mm + topLoad();
}
function fenceH() {
  return Math.max(packH() * 0.62, wall() * 2);
}
function shoulderH() {
  return mmMin(3, 1.2);
}
function shoulderD() {
  return Math.min(innerRaceD() + 4, retainerOd() - 1);
}
function beadH() {
  return mmMin(2.4, 1.2);
}
function beadD() {
  return innerRaceD() + spec.nozzle_mm * 2;
}
function lockFlatX() {
  return innerRaceD() * 0.22;
}
function snapGap() {
  return Math.max(innerRaceD() * 0.28, wall() * 2);
}
function journalD() {
  return innerRaceD();
}
function retainerDHole() {
  return journalD() + spec.fit_slip_mm;
}
function retainerFlatX() {
  return lockFlatX() + spec.fit_slip_mm * 0.5;
}
function journalH() {
  return beadZ() + beadH() + wall() - raceZ();
}
function cageRim() {
  return wall() * 2;
}
function cageOd() {
  return pcd() + rollerLen() + 2 * cageRim();
}
function cageId() {
  // Spacer, not a journal. Looser than the plate bore so the plate takes radial load.
  return plateBore() + 2 * wall();
}
function cageH() {
  return fenceH();
}
function cagePocket() {
  return rollerD() + spec.fit_running_mm;
}
function bedReliefH() {
  return spec.bed_relief_mm;
}
function bedReliefD() {
  return spec.bed_relief_mm;
}
function retainerOd() {
  return Math.max(Math.min(plateBore() + 8, axleFlangeD() - 2), plateBore() + 4);
}
function retainerId() {
  return axleSquare() + spec.fit_slip_mm;
}
function retainerSquare() {
  return axleSquare() + spec.fit_slip_mm;
}
function retainerH() {
  return mmMin(spec.retainer_h, 2);
}
function pcd() {
  // Midline under the blade roots so the couple does not cantilever the plate.
  const underRoot = wingRadius() * 2;
  const maxFit = hubDeckOd() - rollerLen() - 2 * cageRim() - 2;
  const minFit = innerRaceD() + rollerLen() + 2 * wall();
  return Math.max(Math.min(underRoot, maxFit), minFit);
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
  return Math.max(wingH(), hubZ() + hubH() - plateZ());
}
function flangeZ() {
  return 0;
}
function raceZ() {
  return baseH();
}
function raceH() {
  return journalH();
}
function cageZ() {
  return raceZ();
}
function shoulderZ() {
  return plateZ() + hubDeckH() + spec.thrust_float;
}
function beadZ() {
  return retainerZ() + retainerH();
}
function zMid() {
  return cageZ() + packH() * 0.5;
}
function hubZ() {
  return plateZ();
}
function retainerZ() {
  return shoulderZ() + shoulderH();
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
function rollerAngleDeg(index) {
  return (360 / Math.max(spec.roller_count, 1)) * index;
}
function rollerXY(index) {
  const radians = (rollerAngleDeg(index) * Math.PI) / 180;
  const r = pcd() * 0.5;
  return [r * Math.cos(radians), r * Math.sin(radians)];
}
function rollerAxis(index) {
  const radians = (rollerAngleDeg(index) * Math.PI) / 180;
  return [Math.cos(radians), Math.sin(radians), 0];
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
    spec.fit_running_mm + 1e-9 < spec.fit_pip_mm &&
    Math.abs(spec.clearance_mm - spec.fit_running_mm) < 1e-9 &&
    Math.abs(spec.bed_relief_mm - spec.nozzle_mm * 2) < 1e-9 &&
    Math.abs(spec.fit_pip_mm - spec.nozzle_mm * 2) < 1e-9
  );
}
function packOuterR() {
  return pcd() * 0.5 + rollerLen() * 0.5;
}
function rollersOk() {
  const axis0 = rollerAxis(0);
  return (
    spec.roller_count >= 6 &&
    rollerD() + 1e-9 >= spec.roller_min_d &&
    rollerLen() + 1e-9 >= 8 &&
    Math.abs(packH() - rollerD()) < 1e-9 &&
    packOuterR() + 1e-9 >= wingRadius() * 0.9 &&
    pcd() > innerRaceD() + rollerLen() &&
    cageOd() + 1e-9 < hubDeckOd() &&
    Math.abs(plateBore() - (innerRaceD() + spec.fit_running_mm)) < 1e-9 &&
    axleFlangeD() + 1e-9 >= cageOd() &&
    axleFlangeD() + 1e-9 < hubDeckOd() &&
    Math.abs(cagePocket() - (rollerD() + spec.fit_running_mm)) < 1e-9 &&
    fenceH() + 1e-9 < packH() &&
    topLoadPocket() + 1e-9 > cagePocket() &&
    cageId() + 1e-9 > plateBore() &&
    cageRim() + 1e-9 >= wall() * 2 &&
    beadD() + 1e-9 > innerRaceD() &&
    lockFlatX() + 1e-9 < innerRaceD() * 0.5 &&
    Math.abs(axis0[2]) < 1e-9 &&
    Math.abs(axis0[0] - 1) < 1e-9 &&
    cageOd() * 0.5 + 1e-9 >= packOuterR()
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
  return journalH() <= axleFlangeD() && rotorPrintH() > journalH();
}
function stackOk() {
  return (
    Math.abs(raceZ() - baseH()) < 1e-9 &&
    Math.abs(cageZ() - raceZ()) < 1e-9 &&
    Math.abs(plateZ() - (raceZ() + packH() + spec.thrust_float)) < 1e-9 &&
    Math.abs(hubZ() - plateZ()) < 1e-9 &&
    Math.abs(shoulderZ() - (plateZ() + hubDeckH() + spec.thrust_float)) < 1e-9 &&
    Math.abs(retainerZ() - (shoulderZ() + shoulderH())) < 1e-9 &&
    Math.abs(zMid() - (raceZ() + packH() * 0.5)) < 1e-9 &&
    fenceH() + 1e-9 < packH() &&
    bedReliefH() + 1e-9 < hubDeckH() &&
    bedReliefH() + 1e-9 < retainerH() &&
    Math.abs(hubH() - hubDeckH()) < 1e-9 &&
    hubDeckOd() + 1e-9 > axleFlangeD() &&
    hubDeckOd() + 1e-9 >= wingRadius() * 2 &&
    Math.abs(bladeRootZ() - (plateZ() + hubDeckH())) < 1e-9 &&
    Math.abs(packH() - rollerD()) < 1e-9 &&
    hubDeckH() + 1e-9 >= 5 &&
    baseBossD() + 1e-9 < hubDeckOd() &&
    packOuterR() + 1e-9 >= wingRadius() * 0.9 &&
    cageId() + 1e-9 > plateBore() &&
    beadD() + 1e-9 > innerRaceD()
  );
}
function assemblyComponentCount() {
  return 3 + spec.roller_count;
}
function assemblyJointCount() {
  return 2 + spec.roller_count;
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
  const plate =
    Math.PI * ((hubDeckOd() * 0.5) ** 2 - (plateBore() * 0.5) ** 2) * hubDeckH();
  const wings =
    spec.wing_count * 0.62 * ((chordRoot() + chordTip()) * 0.5) * wingThick() * wingH();
  const stator =
    Math.PI * (axleFlangeD() * 0.5) ** 2 * baseH() +
    spec.post_count * ribW() * postCircleR() * baseH() +
    Math.PI * ((cageOd() * 0.5) ** 2 - (cageId() * 0.5) ** 2) * fenceH() +
    Math.PI * (innerRaceD() * 0.5) ** 2 * journalH();
  const rollers = spec.roller_count * Math.PI * (rollerD() * 0.5) ** 2 * rollerLen();
  const retainer =
    (Math.PI * (retainerOd() * 0.5) ** 2 - (innerRaceD() * 0.5) ** 2) * retainerH();
  return (plate + wings + stator + rollers + retainer) / 1000;
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
  return offsetOriginPlane("xy", originZ, z);
}
async function offsetYZ(x) {
  return offsetOriginPlane("yz", originX, x);
}
async function offsetOriginPlane(plane, componentOf, distance) {
  await call("cad_set_focus", { focus: "datums", explicit: true });
  const created = await call("construction_plane_offset", {
    reference: { type: "origin_plane", plane },
    distance,
  });
  const planes = created.planes ?? [];
  let best = null;
  for (const item of planes) {
    const component = componentOf(item.basis);
    if (component == null) continue;
    const err = Math.abs(component - distance);
    if (!best || err < best.err) best = { plane: item, err };
  }
  const hit = best?.plane ?? planes[planes.length - 1];
  if (!hit?.datum_id) throw new Error(`no datum on ${plane} at ${distance}`);
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
async function addDProfile(diameter, flatX) {
  const r = diameter / 2;
  const half = Math.sqrt(Math.max(0, r * r - flatX * flatX));
  const start = Math.atan2(-half, flatX);
  const end = Math.atan2(half, flatX);
  let short = end - start;
  if (short <= 0) short += Math.PI * 2;
  const long = Math.PI * 2 - short;
  const points = [{ x: flatX, y: -half }];
  for (let i = 1; i <= 20; i++) {
    const a = start - (long * i) / 20;
    points.push({ x: r * Math.cos(a), y: r * Math.sin(a) });
  }
  points.push({ x: flatX, y: -half });
  await addPoly(points, true);
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
async function cutBedReliefCircle(z, diameter, bodyId, label) {
  const deck = await offsetXY(z);
  await beginDatum(deck);
  await addCircle(0, 0, diameter);
  const sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: bedReliefH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [bodyId],
    }),
    label,
  );
}
async function cutBedReliefSquare(z, size, bodyId, label) {
  const deck = await offsetXY(z);
  await beginDatum(deck);
  await addOrientedRect([0, 0], size, size, 0);
  const sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: bedReliefH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [bodyId],
    }),
    label,
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

async function buildStator() {
  await beginXY();
  await addCircle(0, 0, axleFlangeD());
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
    "stator race",
  );
  const statorId = newestBody(update, []);
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
        target_body_ids: [statorId],
      }),
      `stator rib ${i}`,
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
        target_body_ids: [statorId],
      }),
      `stator pad ${i}`,
    );
  }
  const raceDeck = await offsetXY(raceZ());
  await beginDatum(raceDeck);
  await addCircle(0, 0, journalD());
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "join",
      extent: { type: "distance", distance: journalH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [statorId],
    }),
    "stator journal",
  );
  await beginDatum(raceDeck);
  await addCircle(0, 0, cageOd());
  await addCircle(0, 0, cageId());
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "join",
      extent: { type: "distance", distance: fenceH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [statorId],
    }),
    "stator fence",
  );
  await beginDatum(await offsetXY(shoulderZ()));
  await addCircle(0, 0, shoulderD());
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "join",
      extent: { type: "distance", distance: shoulderH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [statorId],
    }),
    "stator shoulder",
  );
  await beginDatum(await offsetXY(beadZ()));
  await addCircle(0, 0, beadD());
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "join",
      extent: { type: "distance", distance: beadH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [statorId],
    }),
    "stator snap bead",
  );
  await beginDatum(await offsetXY(retainerZ()));
  await addOrientedRect([lockFlatX() + journalD(), 0], journalD() * 2, journalD() * 2, 0);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: raceZ() + journalH() - retainerZ() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [statorId],
    }),
    "stator D-flat",
  );
  await cutTopLoadSlots(statorId, [statorId]);
  return statorId;
}

async function buildRotor(known) {
  const z0 = plateZ();
  const deck = await offsetXY(z0);
  await beginDatum(deck);
  await addCircle(0, 0, hubDeckOd());
  await addCircle(0, 0, plateBore());
  let sketch = await finishSketch();
  let update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: hubDeckH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "rotor root plate",
  );
  const rotorId = newestBody(update, known);
  for (let index = 0; index < spec.wing_count; index++) {
    await beginDatum(deck);
    await addAirfoil(
      helixCenter(index, 0),
      helixAzimuthDeg(index, 0) + 90,
      chordRoot(),
      spec.airfoil_t_c,
      spec.airfoil_stations,
      teMin(),
    );
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "join",
        extent: { type: "distance", distance: hubDeckH() },
        taper_angle_deg: 0,
        flip: false,
        target_body_ids: [rotorId],
      }),
      `blade root base ${index}`,
    );
    const stations = Math.max(spec.helix_stations, 2);
    const sections = [];
    for (let station = 0; station < stations; station++) {
      const t = station / (stations - 1);
      const chord = chordRoot() * (1 - t) + chordTip() * t;
      await beginDatum(await offsetXY(bladeLoftZ() + wingH() * t));
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
  await cutBedReliefCircle(z0, plateBore() + bedReliefD(), rotorId, "plate bore bed lead-in");
  return rotorId;
}

async function placeRollers(known) {
  const seen = [...known];
  const rollerIds = [];
  for (let i = 0; i < spec.roller_count; i++) {
    const id = await placeRadialCylinder(rollerD(), rollerLen(), i, seen, `roller ${i}`);
    seen.push(id);
    rollerIds.push(id);
  }
  return rollerIds;
}

async function buildRetainer(known) {
  const deck = await offsetXY(retainerZ());
  await beginDatum(deck);
  await addCircle(0, 0, retainerOd());
  let sketch = await finishSketch();
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
  const retainerIdBody = newestBody(update, known);
  await beginDatum(deck);
  await addDProfile(retainerDHole(), retainerFlatX());
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: retainerH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [retainerIdBody],
    }),
    "retainer D-hole",
  );
  await beginDatum(deck);
  await addOrientedRect([-retainerOd() * 0.5, 0], retainerOd(), snapGap(), 0);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: retainerH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [retainerIdBody],
    }),
    "retainer C-gap",
  );
  await cutBedReliefCircle(
    retainerZ(),
    retainerDHole() + bedReliefD(),
    retainerIdBody,
    "retainer D-hole bed lead-in",
  );
  return retainerIdBody;
}

function connectorZ(connector) {
  const origin = connector?.frame?.origin;
  return Array.isArray(origin) ? origin[2] : null;
}

function journalAxisAt(scene, axleId, z, wantRadius) {
  // Prefer the journal cylinder so each revolute can sit at the partner's
  // actual edge Z. Picking the nearest journal circle on a short puck
  // locks cage and plate to different heights and yanks the pack.
  return cylindricalFaceAlong(scene, axleId, [0, 0, z], [0, 0, 1], wantRadius)
    ?? circularEdgeAlong(scene, axleId, [0, 0, z], [0, 0, 1], wantRadius);
}

function axisConnectorAt(scene, bodyId, xy, z, wantRadius) {
  return circularEdgeAt(scene, bodyId, xy, z, wantRadius)
    ?? cylindricalFaceAlong(scene, bodyId, [xy[0], xy[1], z], [0, 0, 1], wantRadius);
}

function radialConnectorAt(scene, bodyId, xy, z, wantRadius, axis) {
  return cylindricalFaceAlong(scene, bodyId, [xy[0], xy[1], z], axis, wantRadius)
    ?? circularEdgeAlong(scene, bodyId, [xy[0], xy[1], z], axis, wantRadius);
}

function axisNorm(v) {
  const n = Math.hypot(v[0], v[1], v[2]);
  return n < 1e-12 ? [0, 0, 1] : [v[0] / n, v[1] / n, v[2] / n];
}
function axisDot(a, b) {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}
function axisAligned(axis, want) {
  return Math.abs(axisDot(axisNorm(axis), axisNorm(want))) >= 0.85;
}
function perpAndAlong(origin, axis, point) {
  const a = axisNorm(axis);
  const d = [point[0] - origin[0], point[1] - origin[1], point[2] - origin[2]];
  const along = axisDot(d, a);
  const closest = [origin[0] + a[0] * along, origin[1] + a[1] * along, origin[2] + a[2] * along];
  const perp = Math.hypot(point[0] - closest[0], point[1] - closest[1], point[2] - closest[2]);
  return { perp, along };
}
function frameAlong(point, primary) {
  const p = axisNorm(primary);
  return {
    origin: [point[0], point[1], point[2]],
    primary_axis: p,
    secondary_axis: Math.abs(p[2]) < 0.5 ? [0, 0, 1] : [1, 0, 0],
  };
}

function cylindricalFaceAlong(scene, bodyId, point, wantAxis, wantRadius) {
  const body = (scene.bodies ?? []).find((item) => item.id === bodyId);
  if (!body) return null;
  let best = null;
  for (const face of body.faces ?? []) {
    const cylinder = face.cylinder;
    if (!cylinder) continue;
    const origin = xyz(cylinder.origin);
    const axis = xyz(cylinder.axis);
    if (!origin || !axis || !axisAligned(axis, wantAxis)) continue;
    const { perp } = perpAndAlong(origin, axis, point);
    if (perp > 3) continue;
    const radius = Number(cylinder.radius);
    if (!Number.isFinite(radius)) continue;
    const radiusErr = Math.abs(radius - wantRadius);
    if (radiusErr > 2.5) continue;
    const score = radiusErr + perp;
    if (!best || score < best.score) best = { face, radius, score };
  }
  if (!best) return null;
  const origin = xyz(best.face.cylinder?.origin) ?? point;
  const frame =
    Math.abs(wantAxis[2]) >= 0.85
      ? { origin: [origin[0], origin[1], point[2]], primary_axis: [0, 0, 1], secondary_axis: [1, 0, 0] }
      : frameAlong(point, wantAxis);
  return {
    body_id: bodyId,
    face_id: best.face.id,
    face_key: best.face.key,
    kind: "cylindrical_face",
    radius: best.radius,
    frame,
  };
}

function circularEdgeAt(scene, bodyId, xy, z, wantRadius) {
  return circularEdgeAlong(scene, bodyId, [xy[0], xy[1], z], [0, 0, 1], wantRadius);
}

function circularEdgeAlong(scene, bodyId, point, wantAxis, wantRadius) {
  const body = (scene.bodies ?? []).find((item) => item.id === bodyId);
  if (!body) return null;
  let best = null;
  for (const edge of body.edges ?? []) {
    const circle = edge.circle;
    if (!circle?.closed) continue;
    const center = xyz(circle.center);
    const normal = xyz(circle.normal);
    if (!center || !normal || !axisAligned(normal, wantAxis)) continue;
    const { perp, along } = perpAndAlong(center, wantAxis, point);
    if (perp > 2) continue;
    const radius = Number(circle.radius);
    if (!Number.isFinite(radius)) continue;
    const radiusErr = Math.abs(radius - wantRadius);
    if (radiusErr > 2.5) continue;
    const score = perp + 0.15 * Math.abs(along) + radiusErr;
    if (!best || score < best.score) best = { edge, center, radius, score };
  }
  if (!best) return null;
  const frame =
    Math.abs(wantAxis[2]) >= 0.85
      ? {
          origin: [best.center[0], best.center[1], best.center[2]],
          primary_axis: [0, 0, 1],
          secondary_axis: [1, 0, 0],
        }
      : frameAlong(point, wantAxis);
  return {
    body_id: bodyId,
    face_id: 0,
    face_key: "",
    edge_id: best.edge.id,
    edge_key: best.edge.key,
    kind: "circular_edge",
    radius: best.radius,
    frame,
  };
}

function authoredOccurrenceId(document, componentId) {
  const matches = (document.component_structure?.occurrences ?? []).filter(
    (occurrence) => occurrence.component_id === componentId,
  );
  if (matches.length === 1) return matches[0].id;
  return (
    matches.find((occurrence) => !String(occurrence.name ?? "").endsWith("_1")) ?? matches[0]
  )?.id;
}

async function requireLinkedSolution() {
  const solution = await call("assembly_solution");
  if (solution.solved !== true) {
    throw new Error(`assembly_solution not solved: ${JSON.stringify(solution.diagnostics ?? [])}`);
  }
  for (const pose of solution.occurrence_poses ?? []) {
    const translation = xyz(pose.translation) ?? [0, 0, 0];
    const yank = Math.hypot(translation[0], translation[1], translation[2]);
    if (yank > 8) {
      throw new Error(
        `occurrence ${pose.occurrence_id} yanked ${yank.toFixed(1)} mm — connectors are off-axis or flipped`,
      );
    }
  }
}

async function createStableJoint(name, kind, connectorA, connectorB, groundedBodyId, flips = [false, true]) {
  let lastError = `${name} failed`;
  for (const flipped of flips) {
    try {
      const joint = await call("assembly_create_joint", {
        name,
        kind,
        connector_a: connectorA,
        connector_b: connectorB,
        flipped,
        grounded_body_id: groundedBodyId,
      });
      try {
        await requireLinkedSolution();
        return;
      } catch (error) {
        const jointId = joint.id ?? joint.joint?.id;
        if (jointId) {
          try {
            await call("assembly_delete_joint", { joint_id: jointId });
          } catch {
            /* retry the other flip */
          }
        }
        lastError = `${name} yanked or failed to solve (flipped=${flipped}): ${error.message ?? error}`;
      }
    } catch (error) {
      lastError = `${name} (flipped=${flipped}): ${error.message ?? error}`;
    }
  }
  throw new Error(lastError);
}

async function formAssembly(ids) {
  await call("cad_set_focus", { focus: "assembly", explicit: true });
  try {
    await call("cad_set_workspace", { workspace: "assembly" });
  } catch {
    /* workspace follow is optional in headless */
  }
  const parts = [
    ["stator", [ids.statorId]],
    ["rotor", [ids.rotorId]],
    ...ids.rollerIds.map((rollerId, index) => [`roller_${index}`, [rollerId]]),
    ["retainer", [ids.retainerId]],
  ];
  let statorOccurrenceId = null;
  for (const [name, bodyIds] of parts) {
    const created = await call("assembly_create_component", {
      name,
      body_ids: bodyIds,
      absorb_promoted_bodies: true,
    });
    const componentId = created.id ?? created.component?.id;
    if (!componentId) throw new Error(`component ${name} missing id: ${JSON.stringify(created)}`);
    const document = await call("assembly_document");
    const occurrenceId = authoredOccurrenceId(document, componentId);
    if (!occurrenceId) throw new Error(`component ${name} has no root occurrence`);
    if (name === "stator") statorOccurrenceId = occurrenceId;
  }
  if (statorOccurrenceId) {
    await call("assembly_set_occurrence_grounded", {
      occurrence_id: statorOccurrenceId,
      grounded: true,
    });
  }
  try {
    await call("assembly_set_grounded_body", { body_id: ids.statorId });
  } catch {
    /* optional on some hosts */
  }
  const scene = await call("solid_scene");
  const need = (connector, label) => {
    if (!connector) throw new Error(label);
    return connector;
  };
  const plateSpin = need(
    axisConnectorAt(scene, ids.rotorId, [0, 0], plateZ() + hubDeckH(), plateBore() * 0.5),
    "no on-axis plate bore for rotor_spin",
  );
  await createStableJoint(
    "rotor_spin",
    "revolute",
    need(
      journalAxisAt(scene, ids.statorId, connectorZ(plateSpin) ?? plateZ() + hubDeckH(), journalD() * 0.5),
      "no on-axis stator journal for rotor_spin",
    ),
    plateSpin,
    ids.statorId,
  );
  for (let index = 0; index < ids.rollerIds.length; index++) {
    const [x, y] = rollerXY(index);
    const z = zMid();
    const axis = rollerAxis(index);
    await createStableJoint(
      `roller_${index}_spin`,
      "revolute",
      need(
        radialConnectorAt(scene, ids.statorId, [x, y], z, topLoadPocket() * 0.5, axis),
        `no stator pocket radial axis for roller ${index}`,
      ),
      need(
        radialConnectorAt(scene, ids.rollerIds[index], [x, y], z, rollerD() * 0.5, axis),
        `no roller radial axis for roller ${index}`,
      ),
      ids.statorId,
    );
  }
  await createStableJoint(
    "retainer_sit",
    "rigid",
    need(
      axisConnectorAt(scene, ids.statorId, [0, 0], retainerZ(), shoulderD() * 0.5),
      "no on-axis stator shoulder for retainer_sit",
    ),
    need(
      axisConnectorAt(scene, ids.retainerId, [0, 0], retainerZ(), retainerOd() * 0.5),
      "no on-axis retainer washer for retainer_sit",
    ),
    ids.statorId,
  );
  if (statorOccurrenceId) {
    try {
      await call("assembly_set_occurrence_grounded", {
        occurrence_id: statorOccurrenceId,
        grounded: true,
      });
    } catch {
      /* stator remains in the grounded cluster */
    }
  }
  const document = await call("assembly_document");
  const defs = document.component_structure?.definitions?.length ?? 0;
  const occs = document.component_structure?.occurrences?.length ?? 0;
  const joints = document.joints?.length ?? 0;
  if (defs !== assemblyComponentCount() || occs !== assemblyComponentCount()) {
    throw new Error(
      `expected ${assemblyComponentCount()} linked parts / occurrences, got ${defs} components / ${occs} occurrences`,
    );
  }
  if (joints < assemblyJointCount()) {
    throw new Error(`expected ≥${assemblyJointCount()} joints, got ${joints}`);
  }
  await requireLinkedSolution();
  return document;
}

async function moveBodies(bodyIds, translation) {
  await transformBodies(bodyIds, translation, [0, 0, 0, 1], [0, 0, 0]);
}
async function transformBodies(bodyIds, translation, rotation, pivot) {
  if (!bodyIds.length) return;
  await call("solid_move_copy", {
    body_ids: bodyIds,
    translation,
    rotation,
    pivot,
    copy: false,
  });
}
function quatAxisAngle(axis, deg) {
  const n = Math.hypot(axis[0], axis[1], axis[2]);
  const a = n < 1e-12 ? [0, 0, 1] : [axis[0] / n, axis[1] / n, axis[2] / n];
  const half = (deg * Math.PI) / 360;
  const s = Math.sin(half);
  return [a[0] * s, a[1] * s, a[2] * s, Math.cos(half)];
}
async function cutTopLoadSlots(statorId, known) {
  const seen = [...known, statorId];
  for (let index = 0; index < spec.roller_count; index++) {
    const toolId = await placeRadialCylinder(
      topLoadPocket(),
      pocketLen(),
      index,
      seen,
      `top-load slot ${index}`,
    );
    requireClean(
      await call("solid_combine", {
        target_body_id: statorId,
        tool_body_ids: [toolId],
        operation: "cut",
        keep_tools: false,
      }),
      `top-load slot cut ${index}`,
    );
  }
}
async function placeRadialCylinder(diameter, length, index, known, label) {
  const x0 = pcd() * 0.5 - length * 0.5;
  const deck = await offsetYZ(x0);
  await beginDatum(deck);
  await addCircle(0, zMid(), diameter);
  const sketch = await finishSketch();
  const update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: length },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    label,
  );
  const id = newestBody(update, known);
  const theta = rollerAngleDeg(index);
  if (Math.abs(theta) > 1e-9) {
    await transformBodies([id], [0, 0, 0], quatAxisAngle([0, 0, 1], theta), [0, 0, zMid()]);
  }
  return id;
}
async function standRoller(rollerId, index) {
  const theta = (rollerAngleDeg(index) * Math.PI) / 180;
  const [x, y] = rollerXY(index);
  await transformBodies(
    [rollerId],
    [0, 0, 0],
    quatAxisAngle([-Math.sin(theta), Math.cos(theta), 0], -90),
    [x, y, zMid()],
  );
}

async function layoutPrintPlate(ids) {
  const gap = 10;
  const rotorR = Math.max(rotorD(), hubDeckOd()) * 0.5;
  const statorR = Math.max(baseEnvelope(), axleFlangeD()) * 0.5;
  const retR = retainerOd() * 0.5;
  const colX = rotorR + gap + statorR;
  await moveBodies([ids.rotorId], [-rotorR - gap * 0.5, 0, -plateZ()]);
  await moveBodies([ids.statorId], [colX, statorR + gap * 0.5, 0]);
  const rollPitch = rollerD() + 4;
  const slotX = colX + statorR + gap + rollerD() * 0.5;
  for (let index = 0; index < ids.rollerIds.length; index++) {
    await standRoller(ids.rollerIds[index], index);
    const [x, y] = rollerXY(index);
    const slotY = -(statorR * 0.4) + index * rollPitch;
    await moveBodies(
      [ids.rollerIds[index]],
      [slotX - x, slotY - y, -(zMid() - rollerLen() * 0.5)],
    );
  }
  await moveBodies([ids.retainerId], [colX, -(statorR + gap + retR), -retainerZ()]);
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
      `FITS  running +${spec.fit_running_mm.toFixed(2)}  PIP +${spec.fit_pip_mm.toFixed(2)}  slip +${spec.fit_slip_mm.toFixed(2)}  friction +${spec.fit_friction_mm.toFixed(2)}  bed lead-in ${spec.bed_relief_mm.toFixed(2)}  (slicer XY hole comp = 0)`,
    ],
    [
      [18, 48],
      "PRINT  one plate, laid out. Rotor STANDING on the root plate. Rollers STANDING (axis Z), assemble lying (axis radial). Others FLAT. PLA Orange + PLA Glow (rotor).",
    ],
    [
      [18, 58],
      "GDT  one stator (Y-frame + race + open fence + journal). Thin flat thrust under the blade roots: stator race = lower, plate underside = upper, radial-axis rollers between. Top-load slots, not PIP. Fence ID looser than the plate bore. Clocked C-snap retainer sits on the journal shoulder — it does not rub the rotor.",
    ],
    [
      [18, 68],
      `BOM  stator (Y-frame + race + fence + D-journal) · rotor (root plate+3×${spec.airfoil}) · ${spec.roller_count} radial rollers · clocked C-snap retainer`,
    ],
    [
      [18, 78],
      `ROLLERS  Ø${rollerD().toFixed(1)} × L${rollerLen().toFixed(1)}  axis radial  PCD ${pcd().toFixed(1)}  pack h=${packH().toFixed(1)}. No metal 608.`,
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
    ["Recovered old nest in the desktop", "Crash recovery reopened the tan/red nine-body kit with orange helix planes. cad_new_project first (0 bodies). Hide datums before save."],
    ["Coincident running faces", "Hub sat on the flange and the retainer sat on the hub. Modeled 0.20 float at every running land."],
    ["Retainer cap through the hub", "Retainer OD covered the hub so the washer looked fused. Washer OD is now between hub bore and hub OD."],
    ["Five colors / five plates", "One laid-out plate. PLA Orange + PLA Glow only."],
    ["Hub as the outer race", "Cage stuffed inside a thin hub wall looked like a colander. The plate is the upper thrust race — not a sleeve the blades hang off."],
    ["Loose bushing sandwich", "A separate orange ring, a postage-stamp flange, and unmatched roller/cage heights. No attach path for an overhung load."],
    ["Washer cup / pancake stack", "Matching an 8 mm land to 8 mm rollers still reads as flat cylinders stacked on the plate. Height-matching flats is not a bearing."],
    ["Tall drum / can on a cracker", "A 28 mm orange tower with webs climbing the wall is a journal you can see from the side. Overturning is a couple across a large PCD, not a tall sleeve. Thin flat thrust under the plate."],
    ["PIP at assembled running clearance", "Cage pockets at +0.40 weld if the roller and cage share a plate. This kit prints rollers standing and drops them in at running +0.40."],
    ["Bed-printed friction bore, no lead-in", "Elephant foot closes a +0.16 locate. 0.80 mm lead-in on every bed-printed hole (plate bore, axle square, retainer square)."],
    ["Race nested around rollers", "Race-to-roller at +0.40 is 0.10 mm/side. They fuse. Print rollers standing, drop them into the cage, then drop the cartridge on the flange."],
    ["Overhung blade roots", "Loft started mid-hub, out at wing radius. First layers of a standing print were air. Root plate is the sit plane; airfoil through the plate; loft from the plate top."],
    ["Tiny ring + blades from the surface above", "Ø41 deck + skinny arms. Airfoils started from the arm top, not a flat sit-plane cut. Root plate out to the blades; loft from plate top so the draft ends on that horizontal."],
    ["Standing-Z pucks", "Ø8 × h3.2 pucks spin about Z. End faces slide on the races. That is not rolling under −Z. Pack height is the roller diameter; axes are radial."],
    ["Tangent-axis rollers", "A tangent axis rolls inward/outward. Relative motion at the race is circumferential, so the roller axis must be radial."],
    ["Rectangular print arms", "Blade roots were rectangular arms + stumps. The airfoil goes through the plate. No spars."],
    ["Inboard pack / cage as journal", "PCD at 58% of the plate left the blade roots cantilevered on 5 mm PLA. Cage ID tighter than the plate bore stole the radial land. Boss tracked the race OD and reprinted a solid orange cylinder. Pack belongs under the blade roots; cage is a spacer; boss only seats the axle."],
    ["Separate axle disk + cage disk", "Two flats that should be one stator. Extra plastic, extra assembly, and a rubbing washer. Merge Y-frame + race + open fence + journal. Top-load the rollers. Clocked C-snap retainer sits on the journal shoulder, not on the rotor."],
  ];
  const usd = estimatedFilamentUsd().toFixed(2);
  const markdown = `# Print Kit Tutor — design report

Spec \`${spec.id}\` · ${spec.title} · nozzle ${spec.nozzle_mm} mm · scale ${spec.scale} of ${spec.printer.name}

## 1. Iteration log

| Iteration | Why it failed / what changed |
|-----------|------------------------------|
${iterations.map(([name, why]) => `| ${name} | ${why} |`).join("\n")}

## 2. Design process

- **Architecture:** Helical H-Darrieus, directionless (no yaw). One printed stator (Y-frame + race + open fence + journal). Thin flat thrust under the plate (large PCD) so the tall blades rotate about Z. No tall mast. No tall drum. No separate axle puck + cage disk.
- **Airfoil:** ${spec.airfoil} (t/c ${spec.airfoil_t_c}). 2026 VAWT dynamic-stall work favors t/c 21–24%. TE blunt to ${teMin()} mm (≥ 2 nozzles). Open drafted tips.
- **Rotor:** one piece — root plate out to the blades (Ø${hubDeckOd().toFixed(1)}), underside is the upper thrust race, plus ${spec.wing_count} helical NACAs lofted from that plate (flat sit-plane cut, chord drafts toward the tip). c=${chordRoot().toFixed(1)}/${chordTip().toFixed(1)} mm, R=${wingRadius().toFixed(1)} mm, span=${wingH().toFixed(1)} mm, helix ${spec.helix_deg}°, σ=${solidity().toFixed(3)}. Envelope/rotor ${(baseEnvelope() / rotorD()).toFixed(2)}.
- **Fits:** assembled running +${spec.fit_running_mm} (rollers on races). Top-load slots are running + two nozzles so rollers drop in from above without support. Same-plate PIP +${spec.fit_pip_mm} is the class; this kit does not PIP the rollers (lying OD would be layers). Clocked C-snap retainer: D-hole + C-gap, slip on the journal neck, sits on the shoulder 0.20 above the plate — it is not a running face. Slicer XY hole compensation stays 0. Plate **sits** 0.20 above the roller pack. Fence height is below pack height so rollers touch both races.
- **Loads:** weight/thrust on the stator race (lower) and the plate underside (upper). Overturning is a couple across the pack **under the blade roots**. The plate bore (running) is the radial land; the fence ID is looser (spacer). Torque about Z stays in the rotor. Centrifugal blade load is taken by the one-piece plate.
- **Friction:** only rolling contacts on the turbine (rollers ↔ races). Plate bore is running, not friction. Fence does not rub the journal. Retainer never rubs the rotor. PLA-on-PLA is a demo; service dry PTFE on the races.
- **Links:** grounded stator; revolute rotor_spin about Z; each roller revolute about its radial axis; rigid retainer_sit on the journal shoulder. assembly_solution must stay solved without yanking parts off-axis.
- **Materials:** ${plaOrange} (stator, rollers, retainer) and ${plaGlow} (rotor). Hardened nozzle for glow. AMS lite is not recommended for glow.
- **Thrust pack:** ${spec.roller_count}× Ø${rollerD().toFixed(1)}×L${rollerLen().toFixed(1)} radial-axis rollers on PCD ${pcd().toFixed(1)} (outer land r=${packOuterR().toFixed(1)}, blade R=${wingRadius().toFixed(1)}) between stator race Ø${axleFlangeD().toFixed(1)} and plate Ø${hubDeckOd().toFixed(1)}. Pack height = roller Ø. Fence ID ${cageId().toFixed(1)} > plate bore ${plateBore().toFixed(1)}. Short journal Ø${innerRaceD().toFixed(1)}×h${raceH().toFixed(1)} centers the plate. Drop rollers into the top-load slots, then the rotor, then snap the retainer. Not a pickup cartridge. Not a tall drum. Not standing-Z pucks.
- **Scale:** source numbers are X2D-max (256×256×260, 8 mm margin). Exam scale ${spec.scale}. Feature floors: roller Ø${spec.roller_min_d}, TE ${spec.airfoil_te_min_mm}, 4-nozzle walls.
- **Service finish:** rotor standing so layer lines run spanwise; sand PLA 400→1000 on skins. Do not vapor-smooth a running fit.
- **Assembly drawing:** A3 sheet, auto-layout, notes for fits / scale / print / BOM.

## 3. Final product

Three printed families, assembly order: ${spec.assembly_order.join(" → ")} (rollers drop into the stator first).

| Part | Count | Role |
|------|------:|------|
| Stator | 1 | Y-frame + lower race + open top-load fence + D-journal + snap bead. Print flat. |
| Rotor | 1 | Root plate (upper thrust race) + 3× ${spec.airfoil} ending on the sit plane. Print standing. |
| Rollers | ${spec.roller_count} | Radial-axis cylinders. Print standing; drop in from above. |
| Retainer | 1 | Clocked C-snap (D-hole + C-gap). Sits on the journal shoulder, not on the rotor. |

Rotor bbox (exam): ${rotorBox ? `${rotorBox.span.map((n) => n.toFixed(1)).join(" × ")} mm` : "n/a"}; faces=${rotorFaces}. Bodies=${bodies.length}.

## 4. Printing cost (plastic / material)

Assumptions: ${spec.filament.name}, ${spec.filament.density_g_cm3} g/cm³, $${spec.filament.price_usd_per_kg}/kg, print-volume factor ${spec.filament.print_volume_factor}.

| | Value |
|--|------:|
| CAD solid (estimate) | ${estimatedSolidCm3().toFixed(1)} cm³ |
| Estimated print mass | ${estimatedPrintMassG().toFixed(1)} g |
| Filament cost | **$${usd}** |

Print plate in \`${out3mfDir}\` (folder wiped first; parts laid out on one plate; rollers print standing; do not print the assembled nest):

${plateFiles.map((file) => `- \`${file}\``).join("\n")}

Slicer: one plate, two materials (PLA Orange + PLA Glow). Stator/retainer flat. Rotor standing on the root plate, tips up. Rollers standing.

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

async function requireBlankDocument() {
  const scene = await call("solid_scene");
  const document = await call("cad_document");
  const bodies = scene.bodies?.length ?? 0;
  const features = document.features?.length ?? 0;
  if (bodies !== 0) {
    throw new Error(
      `cad_new_project left ${bodies} bodies / ${features} features — do not continue a recovered document`,
    );
  }
  return `blank: ${bodies} bodies, ${features} features`;
}

async function hideConstruction() {
  const planes = await call("construction_plane_definitions");
  const planeList = Array.isArray(planes) ? planes : (planes.planes ?? []);
  const datumIds = planeList.map((plane) => plane.datum_id).filter((id) => id != null);
  if (datumIds.length === 0) {
    throw new Error("helix stations created no construction planes to hide");
  }
  const document = await call("cad_document");
  const sketchNames = (document.features ?? [])
    .filter((feature) => feature.kind === "sketch" && feature.name)
    .map((feature) => feature.name);
  const visibility = await call("cad_set_project_visibility", {
    visibility: {
      hidden_body_ids: [],
      hidden_datum_plane_ids: datumIds,
      hidden_sketch_names: sketchNames,
    },
  });
  const hidden = visibility.hidden_datum_plane_ids?.length ?? 0;
  if (hidden === 0) {
    throw new Error("cad_set_project_visibility did not hide construction planes");
  }
  return `hid ${hidden} datums, ${sketchNames.length} sketches`;
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
    !/airfoil|NACA 0021|directionless|cup|design report|service finish|helical|girth|two-land|Y-frame|print plate/i.test(
      recipe,
    )
  ) {
    throw new Error("model_print_kit prompt is missing the 2026 VAWT design contract");
  }
  if (!/blank document|0 bodies|recovered/i.test(recipe)) {
    throw new Error("model_print_kit prompt must start from a blank document");
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
  const blankDetail = await requireBlankDocument();
  await call("cad_set_document_name", { name: spec.document_name });

  const statorId = await buildStator();
  const rotorId = await buildRotor([statorId]);
  const rollerIds = await placeRollers([statorId, rotorId]);
  const retainerId = await buildRetainer([statorId, rotorId, ...rollerIds]);

  let assemblyOk = false;
  let assemblyDetail = "";
  try {
    await formAssembly({ statorId, rotorId, rollerIds, retainerId });
    assemblyOk = true;
    assemblyDetail = `${assemblyComponentCount()} linked parts, ≥${assemblyJointCount()} joints; one stator; radial-axis pack under the blade roots; top-load fence; clocked C-snap retainer; rollers spin about e_r`;
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
    [statorId, plaOrange],
    [rotorId, plaGlow],
    [retainerId, plaOrange],
  ];
  for (const [id, preset] of appearances) {
    await call("set_body_appearance", { body_id: id, preset_id: preset });
  }
  for (const id of rollerIds) {
    await call("set_body_appearance", { body_id: id, preset_id: plaOrange });
  }
  const hideDetail = await hideConstruction();
  const preflight = await call("solid_export_preflight");
  const scene = await call("solid_scene");
  const document = await call("cad_document");
  const assembly = await call("assembly_document").catch(() => ({}));
  try {
    await call("cad_set_workspace", { workspace: "assembly" });
  } catch {
    /* optional */
  }
  const project = await call("cad_project_model");
  mkdirSync(path.dirname(outProject), { recursive: true });
  const projectBytes = writeNbcadArchive(
    typeof project === "string"
      ? project
      : typeof project?.model_json === "string"
        ? project.model_json
        : JSON.stringify(project),
    outProject,
  );
  const removedPlates = cleanKitOutputs();
  await layoutPrintPlate({ statorId, rotorId, rollerIds, retainerId });
  const kitBodies = [statorId, rotorId, retainerId, ...rollerIds];
  const plateFiles = [];
  const plateBytes = [];
  for (const name of currentPlates) {
    const exported = await call("solid_export_3mf", {
      slicer_target: spec.slicer_target,
      include_appearance: true,
      body_ids: kitBodies,
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
  const occurrenceCount = assembly.component_structure?.occurrences?.length ?? 0;
  const jointCount = assembly.joints?.length ?? 0;

  record(report.lessons, "blank", true, `${blankDetail}; ${hideDetail}`);
  record(
    report.lessons,
    "fits",
    fitsOk() && stackOk(),
    `running +${spec.fit_running_mm.toFixed(2)}  PIP +${spec.fit_pip_mm.toFixed(2)}  slip +${spec.fit_slip_mm.toFixed(2)}  friction +${spec.fit_friction_mm.toFixed(2)}  bed lead-in ${spec.bed_relief_mm.toFixed(2)}`,
  );
  record(
    report.lessons,
    "no_press",
    spec.fit_friction_mm > 0 && spec.fit_friction_mm < spec.nozzle_mm,
    "no press: clocked C-snap retainer; plate bore is running; retainer does not rub the rotor",
  );
  record(
    report.lessons,
    "assemble",
    assemblyOk &&
      componentCount === assemblyComponentCount() &&
      occurrenceCount === assemblyComponentCount() &&
      jointCount >= assemblyJointCount() &&
      bodies.length >= spec.min_bodies &&
      rollerIds.length === spec.roller_count,
    `${bodies.length} bodies, ${componentCount} components, ${occurrenceCount} occurrences, ${jointCount} joints; ${assemblyDetail}`,
  );
  record(
    report.lessons,
    "rollers",
    rollersOk() && rollerIds.length === spec.roller_count,
    `${spec.roller_count}× Ø${rollerD().toFixed(1)}×L${rollerLen().toFixed(1)} radial rollers on PCD ${pcd().toFixed(1)}; plate bore ${plateBore().toFixed(1)}; journal Ø${innerRaceD().toFixed(1)}×h${raceH().toFixed(1)}`,
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
    `rotor faces=${rotorFaces} span=${rotorSpan.toFixed(1)} (root plate + 3 blades on the sit plane)`,
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
    `stator prints flat (race Ø${axleFlangeD().toFixed(1)}, fence h${fenceH().toFixed(1)} < pack ${packH().toFixed(1)}); rotor stands on deck ${hubDeckH().toFixed(1)}; rollers print standing, top-load +${topLoad().toFixed(2)}`,
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
    `Spec ${spec.id}  nozzle ${spec.nozzle_mm} mm  scale ${spec.scale}  running +${spec.fit_running_mm} / PIP +${spec.fit_pip_mm} / slip +${spec.fit_slip_mm} / friction +${spec.fit_friction_mm}`,
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
