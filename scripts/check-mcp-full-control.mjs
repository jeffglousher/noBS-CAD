/**
 * Source-derived MCP full-control gate.
 *
 * Fails if a real product surface (host method, DrawingCommand, enabled
 * ribbon action, or File menu item) has no MCP tool mapping. Pointer-only
 * select, disabled placeholders, and UI chrome (theme settings) are skipped.
 */
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];
const fail = (message) => failures.push(message);

const read = (relative) => readFile(path.join(root, relative), 'utf8');

function pascalToSnake(name) {
  const chars = [...name];
  let out = '';
  for (let index = 0; index < chars.length; index += 1) {
    const ch = chars[index];
    if (ch >= 'A' && ch <= 'Z') {
      if (index > 0) {
        const prevLower = chars[index - 1] >= 'a' && chars[index - 1] <= 'z';
        const next = chars[index + 1];
        const nextLower = next >= 'a' && next <= 'z';
        if (prevLower || nextLower) out += '_';
      }
      out += ch.toLowerCase();
    } else {
      out += ch;
    }
  }
  return out;
}

function hostMethods(source) {
  return [...source.matchAll(/^\s+"([a-z0-9_]+)"\s*=>/gm)].map((match) => match[1]);
}

function drawingOps(source) {
  const start = source.indexOf('pub enum DrawingCommand');
  const end = source.indexOf('pub struct DrawingRevisionDraft', start);
  if (start < 0 || end < 0) {
    fail('could not locate DrawingCommand in drawing_ops.rs');
    return [];
  }
  return source
    .slice(start, end)
    .split('\n')
    .flatMap((line) => {
      const trimmed = line.trim();
      const name = trimmed.endsWith(' {},')
        ? trimmed.slice(0, -4)
        : trimmed.endsWith(' {')
          ? trimmed.slice(0, -2)
          : null;
      if (!name || !/^[A-Z][A-Za-z0-9]*$/.test(name)) return [];
      return [pascalToSnake(name)];
    });
}

function matrixEntries(source, constName, field) {
  const start = source.indexOf(`pub const ${constName}`);
  if (start < 0) {
    fail(`missing ${constName} in full_control.rs`);
    return [];
  }
  const slice = source.slice(start, source.indexOf('];', start) + 2);
  return [...slice.matchAll(new RegExp(`${field}:\\s*"([^"]+)"`, 'g'))].map(
    (match) => match[1],
  );
}

function ribbonEnabledFeatures(source) {
  const pointer = new Set(['selectTool']);
  const features = [];
  const seen = new Set();
  for (const match of source.matchAll(/action:\s*'([^']+)'/g)) {
    const action = match[1];
    if (pointer.has(action)) continue;
    const before = source.slice(Math.max(0, match.index - 240), match.index);
    if (!before.includes('enabled: true')) continue;
    const after = source.slice(match.index, match.index + 120);
    const payload = after.match(/payload:\s*'([^']+)'/)?.[1] ?? null;
    const key = `${action}:${payload ?? ''}`;
    if (seen.has(key)) continue;
    seen.add(key);
    features.push({ action, payload });
  }
  return features;
}

function fileMenuFeatures(source) {
  const skip = new Set(['openSettings', 'closeProject']);
  const names = Object.keys(FILE_FN_TO_ID);
  return names.filter((name) => source.includes(name) && !skip.has(name));
}

const FILE_FN_TO_ID = {
  openProject: 'open',
  saveProject: 'save',
  renameProject: 'rename',
  importStep: 'importStep',
  exportStep: 'exportStep',
  export3mf: 'export3mf',
  exportStl: 'exportStl',
  exportActiveDrawingDxf: 'exportDrawingDxf',
  setDrawingProfileExportOpen: 'exportManufacturingProfileDxf',
  newProject: 'new',
};

const [
  hostSrc,
  drawingSrc,
  matrixSrc,
  toolsSrc,
  ribbonSrc,
  fileSrc,
  surfacesSrc,
] = await Promise.all([
  read('crates/sketch/src/host.rs'),
  read('crates/sketch/src/drawing_ops.rs'),
  read('mcp-server/src/full_control.rs'),
  read('mcp-server/src/drawing_tools.rs'),
  read('src/ribbon/config.ts'),
  read('src/components/TopBar.tsx'),
  read('mcp-server/src/surfaces.rs'),
]);

const host = hostMethods(hostSrc);
const internal = new Set(
  [...matrixSrc.matchAll(/INTERNAL_HOST_METHODS[\s\S]*?=\s*&\[([\s\S]*?)\]/g)]
    .flatMap((match) => [...match[1].matchAll(/"([^"]+)"/g)].map((item) => item[1])),
);
const mappedHost = new Set(matrixEntries(matrixSrc, 'HOST_METHODS', 'method'));
for (const method of host) {
  if (internal.has(method)) continue;
  if (!mappedHost.has(method)) {
    fail(`host method ${method} is not in HOST_METHODS`);
  }
}

const ops = drawingOps(drawingSrc);
const toolOps = [...toolsSrc.matchAll(/op:\s*"([^"]+)"/g)].map((match) => match[1]);
const toolOpSet = new Set(toolOps);
for (const op of ops) {
  if (!toolOpSet.has(op)) fail(`DrawingCommand ${op} has no named cad_drawing_* tool`);
}

const ribbonMapped = new Set();
const ribbonBlock = matrixSrc.slice(
  matrixSrc.indexOf('pub const RIBBON_FEATURES'),
  matrixSrc.indexOf('];', matrixSrc.indexOf('pub const RIBBON_FEATURES')) + 2,
);
const ribbonChunks = ribbonBlock.split('RibbonFeature {').slice(1);
for (const chunk of ribbonChunks) {
  const action = chunk.match(/action:\s*"([^"]+)"/)?.[1];
  const payloadMatch = chunk.match(/payload:\s*(?:Some\("([^"]+)"\)|None)/);
  if (!action) continue;
  ribbonMapped.add(`${action}:${payloadMatch?.[1] ?? ''}`);
}
for (const feature of ribbonEnabledFeatures(ribbonSrc)) {
  const key = `${feature.action}:${feature.payload ?? ''}`;
  if (!ribbonMapped.has(key)) {
    fail(`enabled ribbon ${key} is not in RIBBON_FEATURES`);
  }
}

const fileIds = new Set(matrixEntries(matrixSrc, 'FILE_FEATURES', 'id'));
if (fileSrc.includes('setDrawingProfileExportOpen')) {
  if (!fileIds.has('exportManufacturingProfileDxf')) {
    fail('File menu profile DXF is not in FILE_FEATURES');
  }
}
for (const fn of fileMenuFeatures(fileSrc)) {
  const id = FILE_FN_TO_ID[fn];
  if (!id) {
    fail(`File menu handler ${fn} has no FILE_FN_TO_ID mapping`);
    continue;
  }
  if (!fileIds.has(id) && id !== 'save') {
    // save and saveAs share a tool; saveAs is listed separately.
    if (fn === 'saveProject' && (fileIds.has('save') || fileIds.has('saveAs'))) {
      continue;
    }
    fail(`File menu ${fn} (${id}) is not in FILE_FEATURES`);
  }
}
if (!fileIds.has('rename')) fail('FILE_FEATURES is missing rename');

const requiredTools = [
  'cad_invoke',
  'cad_drawing_command',
  'cad_undo',
  'cad_redo',
  'solid_import_step',
  'cad_set_workspace',
];
const mainSrc = await read('mcp-server/src/main.rs');
for (const tool of requiredTools) {
  if (!mainSrc.includes(`"${tool}"`)) {
    fail(`mcp-server is missing tool ${tool}`);
  }
}

function rustStringList(source, constName) {
  const start = source.indexOf(`pub const ${constName}`);
  if (start < 0) {
    fail(`missing ${constName} in surfaces.rs`);
    return [];
  }
  const slice = source.slice(start, source.indexOf('];', start) + 2);
  return [...slice.matchAll(/"([^"]+)"/g)].map((match) => match[1]);
}

const REQUIRED_RESOURCES = [
  'nbcad://document',
  'nbcad://project',
  'nbcad://scene',
  'nbcad://drawing',
  'nbcad://focus',
  'nbcad://workspace',
  'nbcad://sessions',
  'nbcad://sketch',
  'nbcad://sketches',
  'nbcad://profiles',
  'nbcad://visibility',
  'nbcad://appearances',
  'nbcad://materials',
  'nbcad://features',
  'nbcad://assembly',
  'nbcad://assembly_solution',
];
const REQUIRED_PROMPTS = [
  'model_box',
  'model_hole',
  'model_solid',
  'attach_ui',
  'print_3mf',
  'model_print_tool',
  'model_print_kit',
  'import_step',
  'export_step',
  'drawing_read',
  'drawing_sheet',
  'drawing_export',
  'assemble_joint',
  'check_interference',
  'undo_history',
  'invoke',
];

const listedResources = rustStringList(surfacesSrc, 'MAIN_RESOURCE_URIS');
const catalogResources = [
  ...surfacesSrc.matchAll(/resource\("([^"]+)"/g),
].map((match) => match[1]);
for (const uri of REQUIRED_RESOURCES) {
  if (!listedResources.includes(uri)) {
    fail(`MAIN_RESOURCE_URIS is missing ${uri}`);
  }
  if (!catalogResources.includes(uri)) {
    fail(`list_resources is missing ${uri}`);
  }
  if (!surfacesSrc.includes(`"${uri}" => Ok(ResourceKind::`)) {
    fail(`parse_resource_uri is missing ${uri}`);
  }
  if (!mainSrc.includes(`ResourceKind::${resourceKindName(uri)}`)) {
    fail(`read_product_resource is missing ${uri}`);
  }
}

const listedPrompts = rustStringList(surfacesSrc, 'MAIN_PROMPT_NAMES');
const catalogPrompts = [
  ...surfacesSrc.matchAll(/prompt_desc\(\s*"([^"]+)"/g),
].map((match) => match[1]);
for (const name of REQUIRED_PROMPTS) {
  if (!listedPrompts.includes(name)) {
    fail(`MAIN_PROMPT_NAMES is missing ${name}`);
  }
  if (!catalogPrompts.includes(name)) {
    fail(`list_prompts is missing ${name}`);
  }
  if (!surfacesSrc.includes(`"${name}" =>`)) {
    fail(`get_prompt / prompt_title is missing ${name}`);
  }
}

function resourceKindName(uri) {
  const name = uri.slice('nbcad://'.length);
  if (name === 'sketches') return 'Sketches';
  return name
    .split(/[_-]/)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join('');
}

const specSrc = await read('scripts/fixtures/print-kit-tutor.spec.json');
const spec = JSON.parse(specSrc);
if (spec.nozzle_mm !== 0.4 || spec.clearance_mm !== 0.4) {
  fail('print-kit tutor spec must keep a 0.4 mm nozzle clearance stack');
}
if (spec.fit_running_mm !== 0.4) {
  fail('print-kit tutor spec fit_running_mm must be 0.4 (PLA running / roller)');
}
if (spec.fit_pip_mm !== 0.8 || spec.bed_relief_mm !== 0.8) {
  fail('print-kit tutor spec must keep PIP +0.80 and a 0.80 mm bed lead-in (2 nozzles)');
}
if (!(spec.fit_friction_mm < spec.fit_slip_mm && spec.fit_slip_mm < spec.fit_running_mm)) {
  fail('print-kit tutor spec must keep friction < slip < running fits');
}
if (!Array.isArray(spec.print_plates) || spec.print_plates.length !== 1 || spec.print_plates[0] !== '01-kit') {
  fail('print-kit tutor spec must list the one current print plate (01-kit)');
}
if (
  !Array.isArray(spec.retired_print_plates) ||
  !spec.retired_print_plates.includes('02-shaft') ||
  !spec.retired_print_plates.includes('01-base') ||
  !spec.retired_print_plates.includes('06-bushing')
) {
  fail('print-kit tutor spec must list retired plates so the exam can wipe them');
}
if (spec.materials?.orange !== 'bambu.pla.basic.orange' || spec.materials?.glow !== 'bambu.pla.glow.green') {
  fail('print-kit tutor spec must print only PLA orange and PLA glow');
}
const tutorSrc = await read('scripts/mcp-print-kit-tutor.mjs');
if (!tutorSrc.includes('cleanKitOutputs') || !tutorSrc.includes('retired_print_plates')) {
  fail('print-kit Node exam must wipe retired plates before writing the current set');
}
if (!tutorSrc.includes('requireBlankDocument') || !tutorSrc.includes('hideConstruction')) {
  fail('print-kit Node exam must start from a blank document and hide construction planes');
}
if (tutorSrc.includes('name: `${name}_1`') || /assembly_create_occurrence/.test(tutorSrc)) {
  fail('print-kit Node exam must not create a second occurrence of each part');
}
if (!tutorSrc.includes('axisConnectorAt') || !tutorSrc.includes('circularEdgeAt')) {
  fail('print-kit Node exam must mate joints on circular edges or cylinders');
}
if (!tutorSrc.includes('axle_sit') || !tutorSrc.includes('cage_spin') || !tutorSrc.includes('createStableJoint')) {
  fail('print-kit Node exam must link the stator, rotor, cage, and rollers');
}
if (!tutorSrc.includes('rotor_spin') || !tutorSrc.includes('plateBore')) {
  fail('print-kit Node exam must mate the plate bore on the short journal');
}
if (
  tutorSrc.includes('function cupId') ||
  tutorSrc.includes('function cupZ') ||
  tutorSrc.includes('function cupH') ||
  tutorSrc.includes('outerRaceId')
) {
  fail('print-kit Node exam must not keep a tall rotor cup');
}
if (tutorSrc.includes('buildBushing') || tutorSrc.includes('bushing_spin') || tutorSrc.includes('hub_mount')) {
  fail('print-kit Node exam must not keep a loose bushing sandwich (no buildBushing / bushing_spin / hub_mount)');
}
if (!tutorSrc.includes('cutBedReliefCircle') || !tutorSrc.includes('fit_pip_mm') || !tutorSrc.includes('bed_relief')) {
  fail('print-kit Node exam must model PIP clearance and elephant-foot lead-ins');
}
if (!tutorSrc.includes('hubDeckH') || !tutorSrc.includes('bladeRootZ') || !tutorSrc.includes('root plate')) {
  fail('print-kit Node exam must grow blades from a root plate');
}
if (!tutorSrc.includes('plateZ') || !tutorSrc.includes('cageZ') || !tutorSrc.includes('raceH')) {
  fail('print-kit Node exam must stack a thin thrust pack under the plate');
}
if (tutorSrc.includes('Math.max(mm(spec.roller_h), 28)')) {
  fail('print-kit Node exam must not floor the pack at 28 mm (that is a tall drum)');
}
if (tutorSrc.includes('mmMin(spec.roller_h, 3.2)')) {
  fail('print-kit Node exam must not size standing-Z pucks (h3.2 floor)');
}
if (
  !tutorSrc.includes('function packH') ||
  !tutorSrc.includes('function rollerLen') ||
  !tutorSrc.includes('function rollerAxis') ||
  !tutorSrc.includes('radialConnectorAt')
) {
  fail('print-kit Node exam must size a radial-axis thrust pack (pack height = roller Ø)');
}
if (tutorSrc.includes('rootWebInner') || tutorSrc.includes('blade root web')) {
  fail('print-kit Node exam must not climb a cup wall with root webs');
}
if (!tutorSrc.includes('solid_move_copy') || !tutorSrc.includes('layoutPrintPlate')) {
  fail('print-kit Node exam must lay the kit out on one plate before export');
}
if (!tutorSrc.includes('bambu.pla.glow') || !tutorSrc.includes('bambu.pla.basic.orange')) {
  fail('print-kit Node exam must assign PLA orange and PLA glow only');
}
const rustTutor = await read('mcp-server/src/print_kit_tutor.rs');
if (!rustTutor.includes('require_blank_document') || !rustTutor.includes('hide_construction')) {
  fail('print-kit cargo exam must start from a blank document and hide construction planes');
}
if (rustTutor.includes('assembly_create_occurrence')) {
  fail('print-kit cargo exam must not create a second occurrence of each part');
}
if (!rustTutor.includes('axis_connector_at') || !rustTutor.includes('circular_edge_at')) {
  fail('print-kit cargo exam must mate joints on circular edges or cylinders');
}
if (!rustTutor.includes('axle_sit') || !rustTutor.includes('cage_spin') || !rustTutor.includes('create_stable_joint')) {
  fail('print-kit cargo exam must link the stator, rotor, cage, and rollers');
}
if (!rustTutor.includes('rotor_spin') || !rustTutor.includes('plate_bore')) {
  fail('print-kit cargo exam must mate the plate bore on the short journal');
}
if (
  rustTutor.includes('cup_id') ||
  rustTutor.includes('cup_z') ||
  rustTutor.includes('cup_h') ||
  rustTutor.includes('cup_od')
) {
  fail('print-kit cargo exam must not keep a tall rotor cup');
}
if (rustTutor.includes('build_bushing') || rustTutor.includes('bushing_spin') || rustTutor.includes('hub_mount')) {
  fail('print-kit cargo exam must not keep a loose bushing sandwich (no build_bushing / bushing_spin / hub_mount)');
}
if (!rustTutor.includes('cut_bed_relief_circle') || !rustTutor.includes('fit_pip_mm') || !rustTutor.includes('bed_relief')) {
  fail('print-kit cargo exam must model PIP clearance and elephant-foot lead-ins');
}
if (!rustTutor.includes('hub_deck_h') || !rustTutor.includes('blade_root_z') || !rustTutor.includes('root plate')) {
  fail('print-kit cargo exam must grow blades from a root plate');
}
if (!rustTutor.includes('plate_z') || !rustTutor.includes('cage_z') || !rustTutor.includes('race_h')) {
  fail('print-kit cargo exam must stack a thin thrust pack under the plate');
}
if (rustTutor.includes('.max(28.0)') || rustTutor.includes('>= 28.0')) {
  fail('print-kit cargo exam must not floor the pack at 28 mm (that is a tall drum)');
}
if (rustTutor.includes('mm_min(self.roller_h, 3.2)')) {
  fail('print-kit cargo exam must not size standing-Z pucks (h3.2 floor)');
}
if (
  !rustTutor.includes('fn pack_h') ||
  !rustTutor.includes('fn roller_len') ||
  !rustTutor.includes('fn roller_axis') ||
  !rustTutor.includes('radial_connector_at')
) {
  fail('print-kit cargo exam must size a radial-axis thrust pack (pack height = roller Ø)');
}
if (rustTutor.includes('root_web_inner') || rustTutor.includes('blade root web')) {
  fail('print-kit cargo exam must not climb a cup wall with root webs');
}
if (!rustTutor.includes('solid_move_copy') || !rustTutor.includes('layout_print_plate')) {
  fail('print-kit cargo exam must lay the kit out on one plate before export');
}
const printKitPrompt = await read('mcp-server/src/prompts/model_print_kit.md');
if (!/blank document|0 bodies|recovered/i.test(printKitPrompt)) {
  fail('model_print_kit prompt must start from a blank document');
}
if (!/washer cup|pancake/i.test(printKitPrompt)) {
  fail('model_print_kit prompt must still reject a washer/pancake stack');
}
if (!/thin flat thrust/i.test(printKitPrompt)) {
  fail('model_print_kit prompt must require a thin flat thrust bearing');
}
if (!/tall (cup|drum)/i.test(printKitPrompt)) {
  fail('model_print_kit prompt must reject a tall cup or drum');
}
if (!/radial[- ]axis/i.test(printKitPrompt)) {
  fail('model_print_kit prompt must require radial-axis rollers');
}
if (!/standing-Z puck|standing puck/i.test(printKitPrompt)) {
  fail('model_print_kit prompt must reject standing-Z pucks');
}
if (spec.roller_h > 12) {
  fail('print-kit tutor spec roller_h must not be reused as a 70 mm drum height');
}
if (!(spec.roller_len >= 16)) {
  fail('print-kit tutor spec must set roller_len as the scale-1.0 cylinder length');
}
if (!spec.lessons.some((lesson) => lesson.id === 'blank')) {
  fail('print-kit tutor spec must include the blank-document lesson');
}
if (!mainSrc.includes('mod print_kit_tutor')) {
  fail('mcp-server must compile the print-kit tutor exam');
}

if (failures.length > 0) {
  console.error('MCP full-control completeness failed:\n');
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(
  `MCP full-control complete: ${host.length} host methods, ${ops.length} drawing ops, ${ribbonMapped.size} ribbon mappings, ${fileIds.size} file features, ${listedResources.length} resources, ${listedPrompts.length} prompts.`,
);
