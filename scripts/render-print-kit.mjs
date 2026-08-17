/**
 * Load Print-Kit-Tutor.nbcad in the web UI and look at the solid.
 *
 * The print-kit exam is headless. Matching numbers without a 3/4 view is
 * how the 8 mm washer cup shipped. This script is the visual gate:
 * start Vite on :7199 (via `npm run render:print-kit`) and screenshot
 * iso / side / top.
 *
 *   npm run render:print-kit
 *   NBCAD_PRINT_KIT=C:\path\Print-Kit-Tutor.nbcad npm run render:print-kit
 */
import { unzipSync, strFromU8 } from 'fflate';
import { chromium } from 'playwright';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const BASE = 'http://localhost:7199';
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, '..');
const shots = path.join(root, 'docs', 'qa', 'print-kit');
const defaultKit = path.join(os.homedir(), 'Documents', 'noBS-CAD', 'Print-Kit-Tutor.nbcad');
const kitPath = process.env.NBCAD_PRINT_KIT || defaultKit;

const bytes = new Uint8Array(await readFile(kitPath));
const zip = unzipSync(bytes);
const modelBytes = zip['model.json'];
if (!modelBytes) {
  throw new Error(`${kitPath} has no model.json`);
}
const modelJson = strFromU8(modelBytes);

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
const pageErrors = [];
page.on('pageerror', (error) => {
  pageErrors.push(String(error));
  console.log('PAGEERROR:', String(error).slice(0, 400));
});

let failures = 0;
const check = (name, ok, detail = '') => {
  console.log(`  [${ok ? 'ok' : 'FAIL'}] ${name}${ok ? '' : ` — ${detail}`}`);
  if (!ok) failures += 1;
};

try {
  await mkdir(shots, { recursive: true });
  await page.goto(BASE, { waitUntil: 'networkidle' });
  await page.waitForFunction(
    () => window.__appStore?.getState().document !== null && !!window.__engine,
    undefined,
    { timeout: 60_000 },
  );

  const loaded = await page.evaluate(async (json) => {
    const engine = window.__engine;
    const store = window.__appStore.getState();
    const update = await engine.loadProjectModel(json);
    const [
      finishedSketches,
      datumPlanes,
      bodyAppearances,
      drawingDocument,
      assemblyDocument,
      assemblySolution,
      projectVisibility,
    ] = await Promise.all([
      engine.finishedSketches(),
      engine.datumPlaneDefinitions(),
      engine.bodyAppearances(),
      engine.drawingDocument(),
      engine.assemblyDocument(),
      engine.assemblySolution(),
      engine.projectVisibility(),
    ]);
    store.loadProjectState(
      update,
      finishedSketches,
      datumPlanes,
      'Print-Kit-Tutor.nbcad',
      bodyAppearances,
      drawingDocument,
      assemblyDocument,
      projectVisibility,
      assemblySolution,
    );
    if (typeof store.setMode === 'function') store.setMode('solid');
    const errors = update.scene.errors ?? [];
    const bodies = (update.scene.bodies ?? []).map((body) => {
      const positions = body.mesh?.positions ?? [];
      const box = (() => {
        const min = [Infinity, Infinity, Infinity];
        const max = [-Infinity, -Infinity, -Infinity];
        for (let i = 0; i < positions.length; i += 3) {
          for (let axis = 0; axis < 3; axis += 1) {
            min[axis] = Math.min(min[axis], positions[i + axis] ?? 0);
            max[axis] = Math.max(max[axis], positions[i + axis] ?? 0);
          }
        }
        if (!Number.isFinite(min[0])) return null;
        return { min, max, span: [max[0] - min[0], max[1] - min[1], max[2] - min[2]] };
      })();
      const hub = (() => {
        const min = [Infinity, Infinity, Infinity];
        const max = [-Infinity, -Infinity, -Infinity];
        let count = 0;
        for (let i = 0; i < positions.length; i += 3) {
          const x = positions[i];
          const y = positions[i + 1];
          const z = positions[i + 2];
          if (Math.hypot(x, y) > 24) continue;
          count += 1;
          min[0] = Math.min(min[0], x);
          min[1] = Math.min(min[1], y);
          min[2] = Math.min(min[2], z);
          max[0] = Math.max(max[0], x);
          max[1] = Math.max(max[1], y);
          max[2] = Math.max(max[2], z);
        }
        if (count === 0) return null;
        return {
          min,
          max,
          span: [max[0] - min[0], max[1] - min[1], max[2] - min[2]],
          count,
        };
      })();
      return {
        id: body.id,
        name: body.name,
        faces: body.faces?.length ?? 0,
        box,
        hub,
      };
    });
    return {
      bodyCount: bodies.length,
      errors: errors.map((error) => error.message ?? String(error)),
      bodies,
    };
  }, modelJson);

  console.log(`loaded ${kitPath}`);
  console.log(`bodies=${loaded.bodyCount}`);
  if (loaded.errors.length) {
    console.log('scene errors:');
    for (const error of loaded.errors) console.log(`  - ${error}`);
  }
  for (const body of loaded.bodies) {
    const span = body.box?.span?.map((n) => n.toFixed(1)).join(' × ') ?? 'n/a';
    const hubZ = body.hub?.span?.[2];
    console.log(
      `  ${body.name} id=${body.id} faces=${body.faces} span=${span}` +
        (hubZ === undefined ? '' : ` hubZ=${hubZ.toFixed(1)}`),
    );
  }

  check('web UI loaded the kit', loaded.bodyCount >= 5, `bodies=${loaded.bodyCount}`);
  check('WASM recompute has no scene errors', loaded.errors.length === 0, loaded.errors.join('; '));

  const rotor = loaded.bodies.find((body) => /rotor|hub|blade/i.test(body.name)) ?? loaded.bodies[2];
  const hubZ = rotor?.hub?.span?.[2] ?? 0;
  check(
    'rotor hub region is a drum, not a washer',
    hubZ >= 20,
    `hubZ=${hubZ.toFixed(1)} (want ≥20 mm of plate+cup near the axis)`,
  );

  await page.waitForTimeout(400);
  await page.evaluate(() => window.__cameraApi.fit());
  await page.waitForTimeout(400);
  await page.evaluate(() => window.__cameraApi.snapToDirection([1, -1, 0.65]));
  await page.waitForTimeout(400);
  await page.screenshot({ path: path.join(shots, 'iso.png') });

  await page.evaluate(() => window.__cameraApi.snapToDirection([1, 0, 0]));
  await page.waitForTimeout(350);
  await page.evaluate(() => window.__cameraApi.fit());
  await page.waitForTimeout(350);
  await page.screenshot({ path: path.join(shots, 'side.png') });

  await page.evaluate(() => window.__cameraApi.snapToDirection([0, 0, 1]));
  await page.waitForTimeout(350);
  await page.evaluate(() => window.__cameraApi.fit());
  await page.waitForTimeout(350);
  await page.screenshot({ path: path.join(shots, 'top.png') });

  await writeFile(
    path.join(shots, 'scene.json'),
    `${JSON.stringify({ kitPath, pageErrors, ...loaded }, null, 2)}\n`,
  );
  console.log(`shots → ${shots}`);
  check('no page errors', pageErrors.length === 0, pageErrors.join('\n'));
} finally {
  await browser.close();
}

if (failures > 0) {
  process.exit(1);
}
