#!/usr/bin/env node
/**
 * CAD synthesis tutor — rerunnable MCP integration exam.
 *
 * Builds the printed VAWT from
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
import { mkdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
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
const out3mf =
  process.env.NBCAD_3MF_OUT || path.join(defaultKitDir, "Print-Kit-Tutor.3mf");
const outProject =
  process.env.NBCAD_PROJECT_OUT || path.join(defaultKitDir, "Print-Kit-Tutor.nbcad");
const outReport =
  process.env.NBCAD_TUTOR_OUT || path.join(defaultKitDir, "Print-Kit-Tutor-report.json");
const outDesign =
  process.env.NBCAD_DESIGN_OUT || path.join(defaultKitDir, "Print-Kit-Tutor-design.md");

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

async function beginXY() {
  await call("cad_set_focus", { focus: "sketch", explicit: true });
  await call("sketch_begin", { plane: { type: "origin_plane", plane: "xy" } });
  await call("sketch_set_grid_snap", { enabled: false });
}
async function beginXZ() {
  await call("cad_set_focus", { focus: "sketch", explicit: true });
  await call("sketch_begin", { plane: { type: "origin_plane", plane: "xz" } });
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
async function addYFrame(z, height, flip, known, label) {
  const deck = await offsetXY(z);
  await beginDatum(deck);
  await addCircle(0, 0, spec.hub_d);
  let sketch = await finishSketch();
  let update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: height },
      taper_angle_deg: 0,
      flip,
      target_body_ids: [],
    }),
    `${label} hub`,
  );
  const frameId = newestBody(update, known);
  for (let i = 0; i < spec.post_count; i++) {
    const angle = (360 / spec.post_count) * i;
    const [px, py] = postXY(i);
    await beginDatum(deck);
    await addOrientedRect([px * 0.5, py * 0.5], spec.post_circle_r, spec.rib_w, angle);
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "join",
        extent: { type: "distance", distance: height },
        taper_angle_deg: 0,
        flip,
        target_body_ids: [frameId],
      }),
      `${label} rib ${i}`,
    );
    await beginDatum(deck);
    await addCircle(px, py, spec.pad_d);
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "join",
        extent: { type: "distance", distance: height },
        taper_angle_deg: 0,
        flip,
        target_body_ids: [frameId],
      }),
      `${label} pad ${i}`,
    );
  }
  return frameId;
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
function helixAzimuthDeg(index, t) {
  return wingAngleDeg(index) + spec.helix_deg * t;
}
function helixCenter(index, t) {
  const radians = (helixAzimuthDeg(index, t) * Math.PI) / 180;
  return [spec.wing_radius * Math.cos(radians), spec.wing_radius * Math.sin(radians)];
}
function bladeCenter(index) {
  return helixCenter(index, 0);
}
function bladeAngleDeg(index) {
  return helixAzimuthDeg(index, 0) + 90;
}
function rotorD() {
  return bladeTipR() * 2;
}
function frameGirthRatio() {
  return Math.max(spec.top_plate_d, spec.base_d) / rotorD();
}
function postChordRatio() {
  return spec.post_d / spec.wing_chord;
}
function sanityOk() {
  return (
    frameGirthRatio() <= 1.55 &&
    postChordRatio() <= 0.4 &&
    spec.wing_h + 1e-9 >= spec.wing_chord * 2.5 &&
    solidity() >= 0.27 &&
    spec.top_plate_d <= 76 &&
    spec.post_d <= 5.5 &&
    yFrameOk()
  );
}
function yFrameOk() {
  return (
    spec.hub_d + 1e-9 <= 36 &&
    spec.rib_w + 1e-9 <= 6 &&
    spec.hub_d < spec.top_plate_d * 0.55 &&
    spec.pad_d + 1e-9 <= 12
  );
}
function plateStackH() {
  return spec.top_plate_h + spec.plate_boss_h;
}
function bushLd() {
  return spec.bush_h / spec.journal_d;
}
function bushReliefH() {
  return Math.max(0, spec.bush_h - 2 * spec.bush_land_h);
}
function printedBearingOk() {
  return (
    spec.bush_od < 20 &&
    bushLd() + 1e-9 >= 1 &&
    spec.bush_relief_id > spec.bush_id &&
    spec.bush_land_h + 1e-9 >= 1.2 &&
    plateStackH() + 1e-9 >= spec.bush_h + 2 &&
    spec.cone_h >= 3 &&
    spec.thrust_land_od + 1e-9 <= 13
  );
}
function helixOk() {
  return spec.helix_deg >= 45 && spec.helix_stations >= 2;
}
function naca00Thickness(x, thicknessRatio) {
  const t = Math.min(1, Math.max(0, x));
  return (
    5 *
    thicknessRatio *
    (0.2969 * Math.sqrt(t) - 0.126 * t - 0.3516 * t * t + 0.2843 * t ** 3 - 0.1015 * t ** 4)
  );
}
function nacaSymmetricLoop(chord, thicknessRatio, stations, teMin) {
  const count = Math.max(6, stations);
  const xs = [];
  for (let i = 0; i < count; i++) {
    const beta = (Math.PI * i) / (count - 1);
    xs.push(0.5 * (1 - Math.cos(beta)));
  }
  const upper = [];
  const lower = [];
  for (const x of xs) {
    let yt = naca00Thickness(x, thicknessRatio) * chord;
    if (x > 0.85) yt = Math.max(yt, teMin / 2);
    const xc = (x - 0.5) * chord;
    upper.push([xc, yt]);
    lower.push([xc, -yt]);
  }
  const points = [...upper];
  for (let i = lower.length - 2; i >= 0; i--) points.push(lower[i]);
  points.push(points[0]);
  return points;
}
function addAirfoil(center, angleDeg, chord, thicknessRatio, stations, teMin) {
  const angle = (angleDeg * Math.PI) / 180;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const world = nacaSymmetricLoop(chord, thicknessRatio, stations, teMin).map(([x, y]) => ({
    x: center[0] + cos * x - sin * y,
    y: center[1] + sin * x + cos * y,
  }));
  return addPoly(world, true);
}
function solidity() {
  return (spec.wing_count * spec.wing_chord) / (Math.PI * spec.wing_radius * 2);
}
function airfoilOk() {
  return (
    /naca/i.test(spec.airfoil) &&
    spec.airfoil_t_c >= 0.18 &&
    spec.airfoil_t_c <= 0.26 &&
    Math.abs(spec.wing_thick - spec.wing_chord * spec.airfoil_t_c) < 0.15 &&
    spec.airfoil_te_min_mm + 1e-9 >= spec.nozzle_mm * 2
  );
}
function yFrameVolume(height) {
  const hub = Math.PI * (spec.hub_d * 0.5) ** 2 * height;
  const ribLen = Math.max(0, spec.post_circle_r - spec.hub_d * 0.5);
  const ribs = spec.post_count * spec.rib_w * ribLen * height;
  const pads = spec.post_count * Math.PI * (spec.pad_d * 0.5) ** 2 * height;
  return hub + ribs + pads;
}
function estimatedSolidCm3() {
  const base = yFrameVolume(spec.base_h);
  const posts =
    spec.post_count * Math.PI * (spec.post_d * 0.5) ** 2 * postExtrudeH();
  const shaft =
    Math.PI * (spec.journal_d * 0.5) ** 2 * spec.shaft_upper_h +
    Math.PI * (spec.shaft_shoulder_d * 0.5) ** 2 * spec.shaft_shoulder_h;
  const hub =
    Math.PI * ((spec.hub_od * 0.5) ** 2 - (spec.bush_id * 0.5) ** 2) * spec.hub_h;
  const wing = spec.wing_count * 0.7 * spec.wing_chord * spec.wing_thick * spec.wing_h;
  const plate =
    yFrameVolume(spec.top_plate_h) +
    Math.PI * (spec.plate_boss_d * 0.5) ** 2 * spec.plate_boss_h;
  const bushTube =
    Math.PI * ((spec.bush_od * 0.5) ** 2 - (spec.bush_id * 0.5) ** 2) * spec.bush_h;
  const bushRelief =
    Math.PI *
    ((spec.bush_relief_id * 0.5) ** 2 - (spec.bush_id * 0.5) ** 2) *
    bushReliefH();
  const bush = Math.max(0, bushTube - bushRelief);
  const cap =
    Math.PI * ((spec.cap_d * 0.5) ** 2 - (spec.bush_id * 0.5) ** 2) * spec.cap_h;
  return (base + posts + shaft + hub + wing + plate + bush + cap) / 1000;
}
function estimatedPrintMassG() {
  return estimatedSolidCm3() * spec.filament.density_g_cm3 * spec.filament.print_volume_factor;
}
function estimatedFilamentUsd() {
  return (estimatedPrintMassG() / 1000) * spec.filament.price_usd_per_kg;
}
function bladeInnerR() {
  return spec.wing_radius - spec.wing_thick / 2;
}
function bladeTipR() {
  return Math.hypot(spec.wing_radius, spec.wing_chord / 2);
}
function coneR() {
  return spec.cone_h * Math.tan((spec.cone_half_deg * Math.PI) / 180);
}
function coneApexZ() {
  return spec.base_h - spec.cone_h;
}
function shoulderZ() {
  return spec.base_h + spec.shaft_lower_h;
}
function shoulderTop() {
  return shoulderZ() + spec.shaft_shoulder_h;
}
function plateTop() {
  return spec.plate_z + spec.top_plate_h;
}
function bushingZ() {
  return plateTop() - spec.bush_h;
}
function capZ() {
  return plateTop() + spec.cap_float;
}
function shaftTop() {
  return shoulderTop() + spec.shaft_upper_h;
}
function postExtrudeH() {
  return spec.plate_z - spec.base_h + spec.top_plate_h + spec.post_proud;
}
function postInnerR() {
  return spec.post_circle_r - spec.post_d / 2;
}
function postXY(index) {
  const angle = ((360 / spec.post_count) * index * Math.PI) / 180;
  return [spec.post_circle_r * Math.cos(angle), spec.post_circle_r * Math.sin(angle)];
}
function wingAngleDeg(index) {
  return spec.wing_offset_deg + (360 / spec.wing_count) * index;
}
function socketCenter(index) {
  const radians = (wingAngleDeg(index) * Math.PI) / 180;
  const radius = spec.hub_od / 2 - spec.socket_radial / 2;
  return [radius * Math.cos(radians), radius * Math.sin(radians)];
}
function tenonCenter(index) {
  const radians = (wingAngleDeg(index) * Math.PI) / 180;
  const inner = spec.hub_od / 2 - spec.socket_radial + spec.clearance_mm / 2;
  const outer = bladeInnerR() + 0.2;
  const radius = (inner + outer) / 2;
  return [radius * Math.cos(radians), radius * Math.sin(radians)];
}
function tenonRadial() {
  const inner = spec.hub_od / 2 - spec.socket_radial + spec.clearance_mm / 2;
  const outer = bladeInnerR() + 0.2;
  return outer - inner;
}
function socketFloorZ() {
  return shoulderTop() + spec.hub_h - spec.socket_h;
}
function shaftProfile() {
  const journal = spec.journal_d / 2;
  const shoulder = spec.shaft_shoulder_d / 2;
  const land = spec.thrust_land_od / 2;
  const tipZ = coneApexZ() + 0.5;
  const mouthZ = spec.base_h;
  const landZ = spec.base_h + spec.thrust_float;
  const landTop = landZ + spec.thrust_land_h;
  return [
    { x: 0, y: tipZ },
    { x: spec.tip_r, y: tipZ },
    { x: spec.male_cone_r, y: mouthZ },
    { x: spec.male_cone_r, y: landZ },
    { x: land, y: landZ },
    { x: land, y: landTop },
    { x: journal, y: landTop },
    { x: journal, y: shoulderZ() },
    { x: shoulder, y: shoulderZ() },
    { x: shoulder, y: shoulderTop() },
    { x: journal, y: shoulderTop() },
    { x: journal, y: shaftTop() },
    { x: 0, y: shaftTop() },
    { x: 0, y: tipZ },
  ];
}
async function cutFlats(bodyId, z, depth, halfAcross, outer) {
  await beginDatum(await offsetXY(z));
  await call("sketch_add_rectangle", {
    mode: "two_point",
    p1: { x: halfAcross, y: -outer },
    p2: { x: outer, y: outer },
    ctrl_held: false,
  });
  await call("sketch_add_rectangle", {
    mode: "two_point",
    p1: { x: -outer, y: -outer },
    p2: { x: -halfAcross, y: outer },
    ctrl_held: false,
  });
  const sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0, 1],
      operation: "cut",
      extent: { type: "distance", distance: depth },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [bodyId],
    }),
    "double-D flats",
  );
}
function newestBody(update, known) {
  const skip = new Set(known);
  const ids = (update.scene?.bodies ?? []).map((body) => body.id).filter((id) => !skip.has(id));
  if (!ids.length) throw new Error("no new body");
  return Math.max(...ids);
}

function record(lessons, id, pass, detail) {
  lessons.push({ id, pass: !!pass, detail });
}

function writeDesignReport({
  spec: kit,
  bodies,
  wingBox,
  wingFaces,
  solidCm3,
  massG,
  costUsd,
  out3mf: kit3mf,
  outProject: kitProject,
}) {
  const iterations = [
    ["Print-bed scatter", "Parts did not assemble on one axis. Not a machine."],
    ["Colliding spinner", "Rotor swept the posts. Could not rotate."],
    ["Helical C-buckets", "Leftover loft, not a mount, not an airfoil."],
    ["Hoop sector r20–r28", "Concentric C. Concave faces the axis. No net torque."],
    ["Turntable / lazy Susan", "Competent bearings, no wing. The frame had nothing to do."],
    ["Flat plate 12×2.4×32", "A vane, not a 2026 symmetric section. Directionless VAWT needs an airfoil."],
    ["Straight NACA in a Ø90 cage", "Section was right; girth was wrong. Posts Ø8 vs chord 12. Prismatic blades idle most of the rev."],
    [
      `Helical ${kit.airfoil} on a Ø${kit.top_plate_d} cookie`,
      `Section and helix were right; the stand was still a full plate. Short sleeve L/D 0.5 rubbed a full cylinder.`,
    ],
    [
      `Y-frame + two-land sleeve L/D ${(kit.bush_h / kit.journal_d).toFixed(1)}`,
      `Hub Ø${kit.hub_d} + ${kit.rib_w} mm ribs + Ø${kit.pad_d} pads. Sleeve ${kit.bush_h} mm with Ø${kit.bush_relief_id} relief. Fits stay +${kit.clearance_mm}.`,
    ],
  ];
  const usd = costUsd.toFixed(2);
  const markdown = `# Print Kit Tutor — design report

Spec \`${kit.id}\` · ${kit.title} · nozzle ${kit.nozzle_mm} mm · clearance +${kit.clearance_mm} mm

## 1. Iteration log

| Iteration | Why it failed / what changed |
|-----------|------------------------------|
${iterations.map(([name, why]) => `| ${name} | ${why} |`).join("\n")}

## 2. Design process

- **Architecture:** H-Darrieus, directionless (no yaw). Symmetric section because α reverses each rev. Savonius-only is a drag toy. A HAWT section is the wrong physics.
- **Airfoil:** ${kit.airfoil} (t/c ${kit.airfoil_t_c}). 2026 VAWT dynamic-stall work favors t/c 21–24%, xt/c 27.5–35%, reduced LE radius (NACA 0024–4.5/3.5 best in that study). ${kit.airfoil} is the printable stand-in; TE blunt to ${kit.airfoil_te_min_mm} mm (≥ 2 nozzles).
- **Rotor:** helical ${kit.airfoil}, N=${kit.wing_count}, c=${kit.wing_chord} mm, R=${kit.wing_radius} mm, span=${kit.wing_h} mm, helix ${kit.helix_deg}°, σ=${solidity().toFixed(3)}. Frame/rotor ${frameGirthRatio().toFixed(2)}, post/chord ${postChordRatio().toFixed(2)}. Desk Re ~7e3–1.4e4 at 3–5 m/s, TSR ~2.
- **Fits:** every printed-to-printed running/slip interface is +${kit.clearance_mm} mm diametral, including the tenon. No press. No metal 608.
- **Bushings:** printed 45° cup + narrow Ø${kit.thrust_land_od} land (thrust), two-land sleeve Ø${kit.bush_id}/${kit.bush_od} × ${kit.bush_h} (L/D ${(kit.bush_h / kit.journal_d).toFixed(2)}, relief Ø${kit.bush_relief_id}). Do not tighten below +${kit.clearance_mm} — tight FDM binds. PLA-on-PLA is a demo spin.
- **Service finish:** blades printed so layer lines run spanwise; sand PLA 400→1000 on skins (or ABS vapor, not immersion). Do not vapor-smooth a running fit and keep +${kit.clearance_mm}.
- **Later, not this exam:** catalog bearings, higher AR, modified 0024–4.5/3.5, optional adaptive Darrieus–Savonius starter (*Flow* 2026).

## 3. Final product

Nine coaxial bodies, assembly order: ${kit.assembly_order.join(" → ")}.

| Body | Count | Role |
|------|------:|------|
| Base + posts | 1 | Y-frame stand. Cup + 3× Ø${kit.post_d} on R${kit.post_circle_r}. |
| Shaft | 1 | Cone/land thrust, journal, shoulder, double-D. |
| Hub | 1 | Sits on the shoulder. Three sockets. |
| Wing (${kit.airfoil}) | ${kit.wing_count} | Mid-chord R${kit.wing_radius}, chord tangential, tenon drop-in. |
| Top plate | 1 | Y-frame + hanging boss. Posts through. Two-land seat. |
| Bushing | 1 | Two-land printed sleeve, L/D ${(kit.bush_h / kit.journal_d).toFixed(2)}. |
| Cap | 1 | 0.20 float retain. |

Wing bbox (exam): ${wingBox ? `${wingBox.span.map((n) => n.toFixed(1)).join(" × ")} mm` : "n/a"}; faces=${wingFaces}. Bodies=${bodies.length}.

## 4. Printing cost (plastic / material)

Assumptions: ${kit.filament.name}, ${kit.filament.density_g_cm3} g/cm³, $${kit.filament.price_usd_per_kg}/kg, print-volume factor ${kit.filament.print_volume_factor} (3 walls + ~15% gyroid on bulky parts; blades nearer solid — factor is a kit average).

| | Value |
|--|------:|
| CAD solid (estimate) | ${solidCm3.toFixed(1)} cm³ |
| Estimated print mass | ${massG.toFixed(1)} g |
| Filament cost | **$${usd}** |

3MF: \`${kit3mf}\`  
Project: \`${kitProject}\`

Electricity and machine time are not priced. No additional hardware.
`;
  return {
    ok: markdown.includes("Iteration log") && costUsd > 0.05 && /NACA/i.test(kit.airfoil),
    markdown,
  };
}

const report = { ok: false, spec: spec.id, lessons: [], steps };
try {
  await request("server/discover", {});
  const prompt = toolBody(await request("prompts/get", { name: "model_print_kit", arguments: {} }));
  const recipe = prompt?.messages?.[0]?.content?.text ?? "";
  if (!/thrust|cone|clearance|nozzle/i.test(recipe)) {
    throw new Error("model_print_kit prompt is missing the FDM curriculum");
  }
  if (!/airfoil|NACA 0021|directionless|bushing|design report|service finish|helical|girth|two-land|Y-frame/i.test(recipe)) {
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

  let sketch;
  let update;
  const baseId = await addYFrame(0, spec.base_h, false, [], "base");

  await beginXZ();
  await addPoly([
    { x: 0, y: coneApexZ() },
    { x: coneR(), y: spec.base_h },
    { x: 0, y: spec.base_h },
    { x: 0, y: coneApexZ() },
  ]);
  sketch = await finishSketch();
  requireClean(
    await call("solid_revolve", {
      sketch_name: sketch,
      profile_indices: [0],
      axis_origin: { x: 0, y: 0 },
      axis_direction: { x: 0, y: 1 },
      axis_line_entity_id: null,
      angle_deg: 360,
      flip: false,
      operation: "cut",
      target_body_ids: [baseId],
    }),
    "thrust cup",
  );

  await beginXY();
  await addCircle(0, 0, spec.cone_relief_d);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: spec.base_h + 1 },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [baseId],
    }),
    "cone relief",
  );

  await beginDatum(await offsetXY(spec.base_h));
  const [px, py] = postXY(0);
  await addCircle(px, py, spec.post_d);
  sketch = await finishSketch();
  update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: postExtrudeH() },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "post",
  );
  const postId = newestBody(update, [baseId]);
  const patterned = requireClean(
    await call("solid_circular_pattern", {
      body_ids: [postId],
      axis_origin: { x: 0, y: 0, z: 0 },
      axis_direction: { x: 0, y: 0, z: 1 },
      count: spec.post_count,
      total_angle_deg: 360,
    }),
    "even posts",
  );
  const postIds = (patterned.scene?.bodies ?? [])
    .map((body) => body.id)
    .filter((id) => id !== baseId);
  requireClean(
    await call("solid_combine", {
      target_body_id: baseId,
      tool_body_ids: postIds,
      operation: "join",
      keep_tools: false,
    }),
    "join posts",
  );

  await beginXZ();
  await addPoly(shaftProfile());
  sketch = await finishSketch();
  update = requireClean(
    await call("solid_revolve", {
      sketch_name: sketch,
      profile_indices: [0],
      axis_origin: { x: 0, y: 0 },
      axis_direction: { x: 0, y: 1 },
      axis_line_entity_id: null,
      angle_deg: 360,
      flip: false,
      operation: "new_body",
      target_body_ids: [],
    }),
    "shaft",
  );
  const shaftId = newestBody(update, [baseId]);
  await cutFlats(shaftId, shoulderTop(), spec.hub_h + 0.2, spec.drive_across / 2, 8);

  await beginDatum(await offsetXY(shoulderTop()));
  await addCircle(0, 0, spec.hub_od);
  await addCircle(0, 0, spec.bush_id);
  sketch = await finishSketch();
  update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: spec.hub_h },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "hub",
  );
  const hubId = newestBody(update, [baseId, shaftId]);
  await cutFlats(
    hubId,
    shoulderTop(),
    spec.hub_h + 0.2,
    spec.drive_across_hub / 2,
    spec.bush_id / 2 + 1,
  );
  const hubTop = await offsetXY(shoulderTop() + spec.hub_h);
  for (let i = 0; i < spec.wing_count; i++) {
    await beginDatum(hubTop);
    await addOrientedRect(
      socketCenter(i),
      spec.socket_radial + 2,
      spec.socket_w,
      wingAngleDeg(i),
    );
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "cut",
        extent: { type: "distance", distance: spec.socket_h },
        taper_angle_deg: 0,
        flip: true,
        target_body_ids: [hubId],
      }),
      "hub socket",
    );
  }

  const known = [baseId, shaftId, hubId];
  const wingIds = [];
  for (let i = 0; i < spec.wing_count; i++) {
    const angle = wingAngleDeg(i);
    const stations = Math.max(2, spec.helix_stations);
    const sectionNames = [];
    for (let s = 0; s < stations; s++) {
      const t = s / (stations - 1);
      await beginDatum(await offsetXY(shoulderTop() + spec.wing_h * t));
      await addAirfoil(
        helixCenter(i, t),
        helixAzimuthDeg(i, t) + 90,
        spec.wing_chord,
        spec.airfoil_t_c,
        spec.airfoil_stations,
        spec.airfoil_te_min_mm,
      );
      sectionNames.push(await finishSketch());
    }
    update = requireClean(
      await call("solid_loft", {
        sections: sectionNames.map((name) => ({ sketch_name: name, profile_index: 0 })),
        ruled: false,
        operation: "new_body",
        target_body_ids: [],
        continuity: "g1",
      }),
      "helical wing",
    );
    const wingId = newestBody(update, known);
    await beginDatum(await offsetXY(socketFloorZ()));
    await addOrientedRect(tenonCenter(i), tenonRadial(), spec.tenon_w, angle);
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "join",
        extent: { type: "distance", distance: spec.tenon_h },
        taper_angle_deg: 0,
        flip: false,
        target_body_ids: [wingId],
      }),
      "wing tenon",
    );
    known.push(wingId);
    wingIds.push(wingId);
  }

  const plateId = await addYFrame(spec.plate_z, spec.top_plate_h, false, known, "top plate");
  known.push(plateId);
  await beginDatum(await offsetXY(spec.plate_z));
  await addCircle(0, 0, spec.plate_boss_d);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "join",
      extent: { type: "distance", distance: spec.plate_boss_h },
      taper_angle_deg: 0,
      flip: true,
      target_body_ids: [plateId],
    }),
    "plate boss",
  );
  await beginDatum(await offsetXY(plateTop()));
  await addCircle(0, 0, spec.journal_d + spec.clearance_mm);
  for (let i = 0; i < spec.post_count; i++) {
    const [hx, hy] = postXY(i);
    await addCircle(hx, hy, spec.post_hole);
  }
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [...Array(1 + spec.post_count).keys()],
      operation: "cut",
      extent: { type: "distance", distance: plateStackH() + 1 },
      taper_angle_deg: 0,
      flip: true,
      target_body_ids: [plateId],
    }),
    "plate holes",
  );
  await beginDatum(await offsetXY(plateTop()));
  await addCircle(0, 0, spec.bush_seat);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: spec.bush_h },
      taper_angle_deg: 0,
      flip: true,
      target_body_ids: [plateId],
    }),
    "bushing seat",
  );

  await beginDatum(await offsetXY(bushingZ()));
  await addCircle(0, 0, spec.bush_od);
  await addCircle(0, 0, spec.bush_id);
  sketch = await finishSketch();
  update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: spec.bush_h },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "printed bushing",
  );
  const bushId = newestBody(update, known);
  known.push(bushId);
  if (bushReliefH() > 0 && spec.bush_relief_id > spec.bush_id) {
    await beginDatum(await offsetXY(bushingZ() + spec.bush_land_h));
    await addCircle(0, 0, spec.bush_relief_id);
    sketch = await finishSketch();
    requireClean(
      await call("solid_extrude", {
        sketch_name: sketch,
        profile_indices: [0],
        operation: "cut",
        extent: { type: "distance", distance: bushReliefH() },
        taper_angle_deg: 0,
        flip: false,
        target_body_ids: [bushId],
      }),
      "bushing relief",
    );
  }

  await beginDatum(await offsetXY(capZ()));
  await addCircle(0, 0, spec.cap_d);
  await addCircle(0, 0, spec.bush_id);
  sketch = await finishSketch();
  update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: spec.cap_h },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "cap",
  );
  const capId = newestBody(update, known);

  await call("cad_set_focus", { focus: "print", explicit: true });
  const presets = [
    [baseId, "bambu.pla.basic.black"],
    [shaftId, "bambu.pla.basic.jade_white"],
    [hubId, "bambu.pla.basic.green"],
    ...wingIds.map((id) => [id, "bambu.pla.basic.green"]),
    [plateId, "bambu.pla.basic.black"],
    [bushId, "bambu.pla.matte.dark_gray"],
    [capId, "bambu.pla.basic.red"],
  ];
  for (const [id, preset] of presets) {
    await call("set_body_appearance", { body_id: id, preset_id: preset });
  }
  const preflight = await call("solid_export_preflight");
  const exported = await call("solid_export_3mf", {
    slicer_target: spec.slicer_target,
    include_appearance: true,
  });
  const bytes = Buffer.from(exported.bytes_base64, "base64");
  mkdirSync(path.dirname(out3mf), { recursive: true });
  writeFileSync(out3mf, bytes);
  const scene = await call("solid_scene");
  const document = await call("cad_document");
  const project = await call("cad_project_model");
  const modelJson =
    typeof project === "string"
      ? project
      : typeof project?.model_json === "string"
        ? project.model_json
        : JSON.stringify(project);
  mkdirSync(path.dirname(out3mf), { recursive: true });
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
  const wing = bodies.find((body) => body.id === wingIds[0]);
  const nearAxis = (body) => {
    const box = bboxOf(body);
    if (!box) return false;
    const cx = (box.min[0] + box.max[0]) / 2;
    const cy = (box.min[1] + box.max[1]) / 2;
    return Math.hypot(cx, cy) < 50;
  };
  record(
    report.lessons,
    "clearance",
    Math.abs(spec.bush_id - spec.journal_d - spec.clearance_mm) < 1e-9 &&
      Math.abs(spec.bush_seat - spec.bush_od - spec.clearance_mm) < 1e-9 &&
      Math.abs(spec.post_hole - spec.post_d - spec.clearance_mm) < 1e-9 &&
      Math.abs(spec.drive_across_hub - spec.drive_across - spec.clearance_mm) < 1e-9 &&
      Math.abs(spec.socket_w - spec.tenon_w - spec.clearance_mm) < 1e-9,
    `journal ${spec.journal_d} in bush ${spec.bush_id}; tenon ${spec.tenon_w} in socket ${spec.socket_w}`,
  );
  record(
    report.lessons,
    "no_press",
    spec.clearance_mm >= spec.nozzle_mm && spec.bush_id > spec.journal_d,
    "printed interfaces are +0.40 slip; wing drops into a socket",
  );
  const properStack =
    plateStackH() + 1e-9 >= spec.bush_h + 2 &&
    postExtrudeH() + spec.base_h > plateTop() &&
    bladeTipR() + 1 < postInnerR() &&
    spec.thrust_float > 0 &&
    spec.cap_float > 0;
  record(
    report.lessons,
    "assemble",
    bodies.length >= spec.min_bodies &&
      bodies.some((body) => (bboxOf(body)?.max[2] ?? 0) > plateTop() - 1) &&
      bodies.every(nearAxis) &&
      properStack,
    `${bodies.length} coaxial bodies; blade tip r${bladeTipR().toFixed(1)} in post inner r${postInnerR()}; hub sockets`,
  );
  record(
    report.lessons,
    "thrust",
    Math.abs(spec.cone_half_deg - 45) < 1e-9 &&
      spec.cone_h >= 3 &&
      spec.male_cone_r + 0.15 < coneR() &&
      spec.thrust_land_od > spec.journal_d &&
      spec.thrust_float >= 0.2,
    `45° cup r${coneR().toFixed(1)} / male r${spec.male_cone_r}; Ø${spec.thrust_land_od} land float ${spec.thrust_float}`,
  );
  record(
    report.lessons,
    "even",
    spec.post_count === 3 &&
      spec.wing_count === 3 &&
      Math.abs(spec.wing_offset_deg + spec.helix_deg * 0.5 - 60) < 1e-9 &&
      spec.drive_across_hub > spec.drive_across,
    "3 posts at 120°, 3 wings in the bays, double-D drive",
  );
  record(
    report.lessons,
    "printed_bearings",
    printedBearingOk(),
    `two-land sleeve L/D ${bushLd().toFixed(2)}, relief Ø${spec.bush_relief_id}, land ${spec.bush_land_h} mm; plate stack ${plateStackH()} mm; no metal 608`,
  );
  const wingBox = bboxOf(wing);
  const wingXy = wingBox ? Math.max(wingBox.span[0], wingBox.span[1]) : Infinity;
  record(
    report.lessons,
    "not_2d",
    (wing?.faces?.length ?? 0) >= spec.min_rotor_faces &&
      !!wingBox &&
      wingBox.span[2] > spec.wing_h - 1 &&
      wingXy < spec.wing_radius + spec.wing_chord * 0.65 &&
      spec.wing_thick < spec.wing_chord / 3 &&
      spec.wing_h > spec.wing_chord * 2,
    { faces: wing?.faces?.length, bbox: wingBox, xy: wingXy },
  );
  record(
    report.lessons,
    "airfoil",
    airfoilOk() && (wing?.faces?.length ?? 0) >= spec.min_rotor_faces,
    `${spec.airfoil} t/c=${spec.airfoil_t_c} TE≥${spec.airfoil_te_min_mm} mm; solidity ${solidity().toFixed(2)}; faces=${wing?.faces?.length}`,
  );
  record(
    report.lessons,
    "sanity",
    sanityOk(),
    `frame/rotor ${frameGirthRatio().toFixed(2)}; post/chord ${postChordRatio().toFixed(2)}; σ=${solidity().toFixed(2)}; plate Ø${spec.top_plate_d} post Ø${spec.post_d}`,
  );
  record(
    report.lessons,
    "helix",
    helixOk() && (wingBox?.span[2] ?? 0) > spec.wing_h - 1,
    `${spec.helix_deg}° helix, ${spec.helix_stations} stations, span ${wingBox?.span[2]?.toFixed(1)}`,
  );
  const design = writeDesignReport({
    spec,
    bodies,
    wingBox,
    wingFaces: wing?.faces?.length ?? 0,
    solidCm3: estimatedSolidCm3(),
    massG: estimatedPrintMassG(),
    costUsd: estimatedFilamentUsd(),
    out3mf,
    outProject,
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
  record(
    report.lessons,
    "export",
    features.every((feature) => feature.status?.state === "ok") &&
      (preflight.ok === true || (preflight.timeline_errors ?? []).length === 0) &&
      bytes[0] === 0x50 &&
      bytes[1] === 0x4b &&
      bytes.length > 32,
    { bytes: bytes.length, preflight: preflight.ok, path: out3mf },
  );
  report.bodies = bodies.map((body) => ({
    id: body.id,
    faces: body.faces?.length,
    bbox: bboxOf(body),
  }));
  report.export = {
    path: out3mf,
    byte_length: bytes.length,
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
  console.log(`Spec ${spec.id}  nozzle ${spec.nozzle_mm} mm  clearance +${spec.clearance_mm} mm`);
  for (const lesson of spec.lessons) {
    const result = report.lessons.find((item) => item.id === lesson.id);
    const mark = result?.pass ? "PASS" : "FAIL";
    console.log(`\n[${mark}] ${lesson.title}`);
    console.log(`  ${lesson.teach}`);
    if (result?.detail) console.log(`  ${typeof result.detail === "string" ? result.detail : JSON.stringify(result.detail)}`);
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
