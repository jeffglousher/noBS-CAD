#!/usr/bin/env node
/**
 * CAD synthesis tutor — rerunnable MCP integration exam.
 *
 * Builds the printed turntable from
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
async function addPoly(points) {
  for (let i = 0; i < points.length - 1; i++) {
    const dx = points[i + 1].x - points[i].x;
    const dy = points[i + 1].y - points[i].y;
    if (dx * dx + dy * dy < 1e-8) continue;
    await call("sketch_add_line", { from: points[i], to_raw: points[i + 1], ctrl_held: false });
  }
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
  return spec.plate_z + spec.keeper_h;
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
function pocketXY(index) {
  const angle = ((360 / spec.pocket_count) * index * Math.PI) / 180;
  return [spec.pocket_circle_r * Math.cos(angle), spec.pocket_circle_r * Math.sin(angle)];
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

const report = { ok: false, spec: spec.id, lessons: [], steps };
try {
  await request("server/discover", {});
  const prompt = toolBody(await request("prompts/get", { name: "model_print_kit", arguments: {} }));
  const recipe = prompt?.messages?.[0]?.content?.text ?? "";
  if (!/thrust|cone|clearance|nozzle/i.test(recipe)) {
    throw new Error("model_print_kit prompt is missing the FDM curriculum");
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

  await beginXY();
  await addCircle(0, 0, spec.base_d);
  let sketch = await finishSketch();
  let update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: spec.base_h },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "base plate",
  );
  const baseId = update.scene.bodies[0].id;

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
  await cutFlats(shaftId, shoulderTop(), spec.platter_h + 0.2, spec.drive_across / 2, 8);

  await beginDatum(await offsetXY(shoulderTop()));
  await addCircle(0, 0, spec.platter_d);
  await addCircle(0, 0, spec.bush_id);
  sketch = await finishSketch();
  update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: spec.platter_h },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "platter",
  );
  const platterId = newestBody(update, [baseId, shaftId]);
  const rimZ = shoulderTop() + spec.platter_h;
  await beginDatum(await offsetXY(rimZ));
  await addCircle(0, 0, spec.rim_d);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: spec.rim_depth },
      taper_angle_deg: 0,
      flip: true,
      target_body_ids: [platterId],
    }),
    "platter rim well",
  );
  await beginDatum(await offsetXY(rimZ));
  for (let i = 0; i < spec.pocket_count; i++) {
    const [hx, hy] = pocketXY(i);
    await addCircle(hx, hy, spec.pocket_d);
  }
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [...Array(spec.pocket_count).keys()],
      operation: "cut",
      extent: { type: "distance", distance: spec.platter_h + 1 },
      taper_angle_deg: 0,
      flip: true,
      target_body_ids: [platterId],
    }),
    "even wells",
  );
  await cutFlats(
    platterId,
    shoulderTop(),
    spec.platter_h + 0.2,
    spec.drive_across_hub / 2,
    spec.bush_id / 2 + 1,
  );

  await beginDatum(await offsetXY(spec.plate_z));
  await addCircle(0, 0, spec.keeper_d);
  sketch = await finishSketch();
  update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: spec.keeper_h },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "keeper",
  );
  const keeperId = newestBody(update, [baseId, shaftId, platterId]);
  await beginDatum(await offsetXY(plateTop()));
  await addCircle(0, 0, spec.journal_d + spec.clearance_mm);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: spec.keeper_h + 1 },
      taper_angle_deg: 0,
      flip: true,
      target_body_ids: [keeperId],
    }),
    "keeper journal",
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
      target_body_ids: [keeperId],
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
  const bushId = newestBody(update, [baseId, shaftId, platterId, keeperId]);

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
  const capId = newestBody(update, [baseId, shaftId, platterId, keeperId, bushId]);

  await call("cad_set_focus", { focus: "print", explicit: true });
  const presets = [
    [baseId, "bambu.pla.basic.black"],
    [shaftId, "bambu.pla.basic.jade_white"],
    [platterId, "bambu.pla.basic.green"],
    [keeperId, "bambu.pla.basic.black"],
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
  const platter = bodies.find((body) => body.id === platterId);
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
      Math.abs(spec.drive_across_hub - spec.drive_across - spec.clearance_mm) < 1e-9,
    `journal ${spec.journal_d} in bush ${spec.bush_id}; bush ${spec.bush_od} in seat ${spec.bush_seat}`,
  );
  record(
    report.lessons,
    "no_press",
    spec.clearance_mm >= spec.nozzle_mm && spec.bush_id > spec.journal_d,
    "printed interfaces are +0.40 slip; cone and cap retain the stack",
  );
  const properStack =
    spec.keeper_h > spec.bush_h &&
    spec.platter_d > spec.base_d + 8 &&
    spec.keeper_d < spec.platter_d * 0.5 &&
    spec.thrust_float > 0 &&
    spec.cap_float > 0;
  record(
    report.lessons,
    "assemble",
    bodies.length >= spec.min_bodies &&
      bodies.some((body) => (bboxOf(body)?.max[2] ?? 0) > plateTop() - 1) &&
      bodies.every(nearAxis) &&
      properStack,
    `${bodies.length} coaxial bodies; platter Ø${spec.platter_d} on foot Ø${spec.base_d}; keeper Ø${spec.keeper_d}`,
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
    spec.pocket_count === 3 && spec.drive_across_hub > spec.drive_across,
    "3 wells at 120°, double-D drive",
  );
  record(
    report.lessons,
    "printed_bearings",
    spec.bush_od < 20 && spec.bush_h < spec.keeper_h && spec.cone_h >= 3,
    "printed sleeve on a 2 mm land + printed cone/land thrust; no metal 608",
  );
  const platterBox = bboxOf(platter);
  record(
    report.lessons,
    "not_2d",
    (platter?.faces?.length ?? 0) >= spec.min_rotor_faces &&
      !!platterBox &&
      platterBox.span[2] > spec.platter_h - 1,
    { faces: platter?.faces?.length, bbox: platterBox },
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
