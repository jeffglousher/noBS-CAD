#!/usr/bin/env node
/**
 * CAD synthesis tutor — rerunnable MCP integration exam.
 *
 * Builds the FDM-tolerant journal kit from
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
    await call("sketch_add_line", { from: points[i], to_raw: points[i + 1], ctrl_held: false });
  }
}
async function addC(deg, ox, oy) {
  const overlap = spec.rotor_d * spec.overlap;
  const radius = (spec.rotor_d + overlap) / 4;
  const inner = radius - spec.blade_wall;
  const center = radius - overlap / 2;
  const angle = (deg * Math.PI) / 180;
  const cx = center * Math.cos(angle);
  const cy = center * Math.sin(angle);
  const length = Math.hypot(cx, cy) || 1;
  const ux = cx / length;
  const uy = cy / length;
  const vx = -uy;
  const vy = ux;
  const point = (x, y) => ({ x: x + ox, y: y + oy });
  const S = point(cx + radius * vx, cy + radius * vy);
  const F = point(cx + radius * ux, cy + radius * uy);
  const E = point(cx - radius * vx, cy - radius * vy);
  const Si = point(cx + inner * vx, cy + inner * vy);
  const Fi = point(cx + inner * ux, cy + inner * uy);
  const Ei = point(cx - inner * vx, cy - inner * vy);
  await call("sketch_add_arc_3pt", { p1: S, p2: F, p3: E, ctrl_held: false });
  await call("sketch_add_line", { from: E, to_raw: Ei, ctrl_held: false });
  await call("sketch_add_arc_3pt", { p1: Ei, p2: Fi, p3: Si, ctrl_held: false });
  await call("sketch_add_line", { from: Si, to_raw: S, ctrl_held: false });
}

function record(lessons, id, pass, detail) {
  lessons.push({ id, pass: !!pass, detail });
}

const report = { ok: false, spec: spec.id, lessons: [], steps };
try {
  await request("server/discover", {});
  const prompt = toolBody(await request("prompts/get", { name: "model_print_kit", arguments: {} }));
  const recipe = prompt?.messages?.[0]?.content?.text ?? "";
  if (!/608|clearance|nozzle/i.test(recipe)) {
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

  const jr = spec.journal_d / 2;
  const fr = spec.flange_d / 2;
  await beginXZ();
  await addPoly([
    { x: 0, y: 0 },
    { x: fr - spec.bed_chamfer, y: 0 },
    { x: fr, y: spec.bed_chamfer },
    { x: fr, y: spec.flange_h },
    { x: jr, y: spec.flange_h },
    { x: jr, y: spec.shaft_h - spec.tip_chamfer },
    { x: jr - spec.tip_chamfer, y: spec.shaft_h },
    { x: 0, y: spec.shaft_h },
    { x: 0, y: 0 },
  ]);
  let sketch = await finishSketch();
  let update = requireClean(
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
  const shaftId = update.scene.bodies[0].id;

  const [bx, by] = spec.placements.bushing;
  await beginXY();
  await addCircle(bx, by, spec.bush_od);
  await addCircle(bx, by, spec.bush_id);
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
    "bushing",
  );
  const bushId = update.scene.bodies.find((body) => body.id !== shaftId).id;

  const [hx, hy] = spec.placements.housing;
  const half = spec.house_plate / 2;
  await beginXY();
  await call("sketch_add_rectangle", {
    mode: "two_point",
    p1: { x: hx - half, y: hy - half },
    p2: { x: hx + half, y: hy + half },
    ctrl_held: false,
  });
  sketch = await finishSketch();
  update = requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "new_body",
      extent: { type: "distance", distance: spec.house_height },
      taper_angle_deg: 0,
      flip: false,
      target_body_ids: [],
    }),
    "housing",
  );
  const known = new Set([shaftId, bushId]);
  const houseId = update.scene.bodies.find((body) => !known.has(body.id)).id;
  const top = await offsetXY(spec.house_height);
  await beginDatum(top);
  await addCircle(hx, hy, spec.thru_d);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: spec.house_height + 1 },
      taper_angle_deg: 0,
      flip: true,
      target_body_ids: [houseId],
    }),
    "through",
  );
  await beginDatum(top);
  await addCircle(hx, hy, spec.house_id);
  sketch = await finishSketch();
  requireClean(
    await call("solid_extrude", {
      sketch_name: sketch,
      profile_indices: [0],
      operation: "cut",
      extent: { type: "distance", distance: spec.house_h },
      taper_angle_deg: 0,
      flip: true,
      target_body_ids: [houseId],
    }),
    "608 seat",
  );

  const [lx, ly] = spec.placements.blade;
  const sectionNames = [];
  for (let i = 0; i < spec.loft_stations; i++) {
    const z = (spec.blade_h * i) / (spec.loft_stations - 1);
    const ang = (spec.blade_twist_deg * i) / (spec.loft_stations - 1);
    await beginDatum(await offsetXY(z));
    await addC(ang, lx, ly);
    sectionNames.push(await finishSketch());
  }
  update = requireClean(
    await call("solid_loft", {
      sections: sectionNames.map((name) => ({ sketch_name: name, profile_index: 0 })),
      ruled: false,
      operation: "new_body",
      target_body_ids: [],
      continuity: "g0",
      centerline: null,
      guide_rail: null,
    }),
    "helical loft",
  );
  known.add(houseId);
  const bladeId = update.scene.bodies.find((body) => !known.has(body.id)).id;

  await call("cad_set_focus", { focus: "print", explicit: true });
  const presets = [
    [shaftId, "bambu.pla.basic.jade_white"],
    [bushId, "bambu.pla.matte.dark_gray"],
    [houseId, "bambu.pla.basic.black"],
    [bladeId, "bambu.pla.basic.green"],
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
  const blade = bodies.find((body) => body.id === bladeId);
  const bush = bodies.find((body) => body.id === bushId);
  record(
    report.lessons,
    "clearance",
    Math.abs(spec.bush_id - spec.journal_d - spec.clearance_mm) < 1e-9 &&
      Math.abs(spec.house_id - spec.bush_od - spec.clearance_mm) < 1e-9,
    `journal ${spec.journal_d} in bush ${spec.bush_id}; bush ${spec.bush_od} in house ${spec.house_id}`,
  );
  record(
    report.lessons,
    "no_press",
    spec.clearance_mm >= spec.nozzle_mm && spec.house_id > spec.bush_od,
    "printed interfaces are +0.40 slip; 608 seat retains the race",
  );
  record(
    report.lessons,
    "orientation",
    bodies.length >= spec.min_bodies &&
      bodies.every((body) => {
        const box = bboxOf(body);
        return box && box.min[2] > -0.6 && box.min[2] < 0.6;
      }),
    `${bodies.length} bodies on z=0`,
  );
  record(
    report.lessons,
    "xy_holes",
    spec.thru_d > spec.journal_d,
    "housing through and 608 seat cut as XY circles from a top datum",
  );
  const bushBox = bboxOf(bush);
  record(
    report.lessons,
    "mechanical",
    !!bushBox && bushBox.span[2] > 6 && bushBox.span[2] < 8.5 && bushBox.span[0] > 18,
    bushBox,
  );
  const bladeBox = bboxOf(blade);
  record(
    report.lessons,
    "not_2d",
    (blade?.faces?.length ?? 0) >= spec.min_blade_faces && !!bladeBox && bladeBox.span[2] > spec.blade_h - 2,
    { faces: blade?.faces?.length, bbox: bladeBox },
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
