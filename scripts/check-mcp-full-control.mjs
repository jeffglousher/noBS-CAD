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
if (Math.abs(spec.bush_id - spec.journal_d - spec.clearance_mm) > 1e-9) {
  fail('print-kit tutor spec bush_id must be journal + clearance');
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
