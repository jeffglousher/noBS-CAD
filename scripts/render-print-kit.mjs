/**
 * Load Print-Kit-Tutor.nbcad in the web UI and look at the solid.
 *
 * The print-kit exam is headless. Matching numbers without a 3/4 view is
 * how the 8 mm washer cup and the 28 mm orange drum shipped. This script
 * is the visual gate: load the kit in the web UI (WASM), then paint the
 * tessellation. The hub must be a thin plate over a flat thrust pack.
 * The browser shell no longer owns WebGL — Bevy paints on desktop only —
 * so the screenshots are an agent canvas of the same meshes.
 *
 *   npm run render:print-kit
 *   NBCAD_PRINT_KIT=C:\path\Print-Kit-Tutor.nbcad npm run render:print-kit
 */
import { unzipSync, strFromU8 } from 'fflate';
import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const PORT = 7199;
const BASE = `http://127.0.0.1:${PORT}`;
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, '..');
const shots = path.join(root, 'docs', 'qa', 'print-kit');
const defaultKit = path.join(os.homedir(), 'Documents', 'noBS-CAD', 'Print-Kit-Tutor.nbcad');
const kitPath = process.env.NBCAD_PRINT_KIT || defaultKit;

async function serverUp() {
  try {
    const res = await fetch(`${BASE}/`);
    return res.ok;
  } catch {
    return false;
  }
}

async function ensureDevServer() {
  if (await serverUp()) return null;
  const viteJs = path.join(root, 'node_modules', 'vite', 'bin', 'vite.js');
  const server = spawn(
    process.execPath,
    [viteJs, '--host', '127.0.0.1', '--port', String(PORT), '--strictPort'],
    {
      cwd: root,
      stdio: 'ignore',
      detached: true,
      windowsHide: true,
    },
  );
  for (let i = 0; i < 90; i += 1) {
    if (await serverUp()) return server;
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(`dev server did not come up on port ${PORT}`);
}

const bytes = new Uint8Array(await readFile(kitPath));
const zip = unzipSync(bytes);
const modelBytes = zip['model.json'];
if (!modelBytes) {
  throw new Error(`${kitPath} has no model.json`);
}
const modelJson = strFromU8(modelBytes);
const started = await ensureDevServer();

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
  const consoleLines = [];
  page.on('console', (msg) => {
    const line = `${msg.type()}: ${msg.text()}`;
    consoleLines.push(line);
    if (msg.type() === 'error') console.log('CONSOLE', line.slice(0, 300));
  });
  await page.goto(BASE, { waitUntil: 'load', timeout: 120_000 });
  try {
    await page.waitForFunction(
      () => window.__appStore?.getState().document !== null && !!window.__engine,
      undefined,
      { timeout: 90_000 },
    );
  } catch (error) {
    const probe = await page.evaluate(() => ({
      href: location.href,
      hasStore: !!window.__appStore,
      hasEngine: !!window.__engine,
      document: window.__appStore?.getState()?.document?.name ?? null,
      title: document.title,
      bodyText: document.body?.innerText?.slice(0, 400) ?? '',
    }));
    console.log('boot probe', JSON.stringify(probe, null, 2));
    await mkdir(shots, { recursive: true });
    await page.screenshot({ path: path.join(shots, 'boot-fail.png') });
    throw error;
  }

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
    'rotor hub is a thin plate, not a tall drum',
    hubZ >= 4.5 && hubZ <= 8,
    `hubZ=${hubZ.toFixed(1)} (want ~5 mm plate; ≥20 mm is a tall drum)`,
  );

  // The browser shell no longer owns WebGL — Bevy paints only on desktop.
  // Draw the WASM tessellation ourselves so the thrust pack is actually looked at.
  const paint = async (name, eye, focus = 'all') => {
    await page.evaluate(({ direction, focus: mode }) => {
      const state = window.__appStore.getState();
      const bodies = state.solidScene.bodies ?? [];
      const appearances = state.bodyAppearances ?? [];
      const poses = new Map(
        (state.assemblySolution?.instance_body_poses ?? state.assemblySolution?.body_poses ?? [])
          .map((pose) => [pose.body_id, pose]),
      );
      let canvas = document.getElementById('agent-mesh-preview');
      if (!canvas) {
        canvas = document.createElement('canvas');
        canvas.id = 'agent-mesh-preview';
        canvas.style.cssText = 'position:fixed;inset:0;z-index:9999;background:#d5dbe2;';
        document.body.appendChild(canvas);
      }
      const width = (canvas.width = 1440);
      const height = (canvas.height = 900);
      const ctx = canvas.getContext('2d');
      ctx.fillStyle = '#d5dbe2';
      ctx.fillRect(0, 0, width, height);

      const colorFor = (body) => {
        const appearance = appearances.find((item) => item.body_id === body.id);
        const name = `${appearance?.material_name ?? ''} ${appearance?.color_name ?? ''}`.toLowerCase();
        if (name.includes('glow') || name.includes('green')) return [92, 176, 98];
        if (appearance?.color) {
          return [appearance.color.r, appearance.color.g, appearance.color.b];
        }
        return [224, 112, 56];
      };

      const applyPose = (pose, x, y, z) => {
        const t = pose?.translation ?? [0, 0, 0];
        const q = pose?.rotation;
        if (!q || q.length !== 4) return [x + (t[0] ?? 0), y + (t[1] ?? 0), z + (t[2] ?? 0)];
        const [qx, qy, qz, qw] = q;
        const tx = 2 * (qy * z - qz * y);
        const ty = 2 * (qz * x - qx * z);
        const tz = 2 * (qx * y - qy * x);
        return [
          x + qw * tx + (qy * tz - qz * ty) + (t[0] ?? 0),
          y + qw * ty + (qz * tx - qx * tz) + (t[1] ?? 0),
          z + qw * tz + (qx * ty - qy * tx) + (t[2] ?? 0),
        ];
      };

      const triangles = [];
      const worldMin = [Infinity, Infinity, Infinity];
      const worldMax = [-Infinity, -Infinity, -Infinity];
      const rotorId = bodies.reduce((best, body) => {
        const z = (body.mesh?.positions ?? []).reduce((maxZ, _value, index, positions) => {
          if (index % 3 !== 2) return maxZ;
          return Math.max(maxZ, positions[index] ?? 0);
        }, 0);
        if (!best || z > best.z) return { id: body.id, z };
        return best;
      }, null)?.id;
      const baseId = bodies.reduce((best, body) => {
        const z = (body.mesh?.positions ?? []).reduce((minZ, _value, index, positions) => {
          if (index % 3 !== 2) return minZ;
          return Math.min(minZ, positions[index] ?? 0);
        }, Infinity);
        if (!best || z < best.z) return { id: body.id, z };
        return best;
      }, null)?.id;
      const axleId = bodies.reduce((best, body) => {
        if (body.id === baseId || body.id === rotorId) return best;
        const zs = [];
        const xs = [];
        const positions = body.mesh?.positions ?? [];
        for (let i = 0; i < positions.length; i += 3) {
          xs.push(Math.hypot(positions[i] ?? 0, positions[i + 1] ?? 0));
          zs.push(positions[i + 2] ?? 0);
        }
        const spanZ = (zs.length ? Math.max(...zs) - Math.min(...zs) : 0);
        const spanXY = xs.length ? Math.max(...xs) * 2 : 0;
        if (spanZ < 8) return best;
        if (!best || spanXY > best.spanXY) return { id: body.id, spanXY };
        return best;
      }, null)?.id;
      for (const body of bodies) {
        if (mode === 'pack' && body.id === rotorId) continue;
        if (mode === 'underpack' && body.id === baseId) continue;
        if (mode === 'races' && (body.id === baseId || body.id === axleId)) continue;
        const positions = body.mesh?.positions ?? [];
        const indices = body.mesh?.indices ?? [];
        const pose = poses.get(body.id);
        if (pose?.visible === false) continue;
        const rgb = colorFor(body);
        const pushVertex = (index) => {
          const x = positions[index * 3] ?? 0;
          const y = positions[index * 3 + 1] ?? 0;
          const z = positions[index * 3 + 2] ?? 0;
          return applyPose(pose, x, y, z);
        };
        if (indices.length >= 3) {
          for (let i = 0; i < indices.length; i += 3) {
            const a = pushVertex(indices[i]);
            const b = pushVertex(indices[i + 1]);
            const c = pushVertex(indices[i + 2]);
            if (mode === 'hub') {
              const x = (a[0] + b[0] + c[0]) / 3;
              const y = (a[1] + b[1] + c[1]) / 3;
              const z = (a[2] + b[2] + c[2]) / 3;
              if (Math.hypot(x, y) > 42 || z > 28) continue;
            }
            for (const point of [a, b, c]) {
              for (let axis = 0; axis < 3; axis += 1) {
                worldMin[axis] = Math.min(worldMin[axis], point[axis]);
                worldMax[axis] = Math.max(worldMax[axis], point[axis]);
              }
            }
            triangles.push({ a, b, c, rgb });
          }
        }
      }
      const center = worldMin.map((value, axis) => (value + worldMax[axis]) / 2);
      const radius = Math.max(
        1,
        ...worldMax.map((value, axis) => Math.abs(value - center[axis])),
      );
      const eyeLen = Math.hypot(...direction) || 1;
      const ez = direction.map((value) => value / eyeLen);
      const upGuess = Math.abs(ez[2]) > 0.9 ? [0, 1, 0] : [0, 0, 1];
      const ex = [
        upGuess[1] * ez[2] - upGuess[2] * ez[1],
        upGuess[2] * ez[0] - upGuess[0] * ez[2],
        upGuess[0] * ez[1] - upGuess[1] * ez[0],
      ];
      const exLen = Math.hypot(...ex) || 1;
      for (let i = 0; i < 3; i += 1) ex[i] /= exLen;
      const ey = [
        ez[1] * ex[2] - ez[2] * ex[1],
        ez[2] * ex[0] - ez[0] * ex[2],
        ez[0] * ex[1] - ez[1] * ex[0],
      ];
      const light = [0.45, -0.35, 0.82];
      const lightLen = Math.hypot(...light);
      for (let i = 0; i < 3; i += 1) light[i] /= lightLen;
      const scale = (Math.min(width, height) * 0.42) / radius;
      const project = (point) => {
        const dx = point[0] - center[0];
        const dy = point[1] - center[1];
        const dz = point[2] - center[2];
        return {
          x: width / 2 + (dx * ex[0] + dy * ex[1] + dz * ex[2]) * scale,
          y: height / 2 - (dx * ey[0] + dy * ey[1] + dz * ey[2]) * scale,
          depth: dx * ez[0] + dy * ez[1] + dz * ez[2],
        };
      };
      const drawn = triangles
        .map((triangle) => {
          const a = project(triangle.a);
          const b = project(triangle.b);
          const c = project(triangle.c);
          const ux = triangle.b[0] - triangle.a[0];
          const uy = triangle.b[1] - triangle.a[1];
          const uz = triangle.b[2] - triangle.a[2];
          const vx = triangle.c[0] - triangle.a[0];
          const vy = triangle.c[1] - triangle.a[1];
          const vz = triangle.c[2] - triangle.a[2];
          const nx = uy * vz - uz * vy;
          const ny = uz * vx - ux * vz;
          const nz = ux * vy - uy * vx;
          const nLen = Math.hypot(nx, ny, nz) || 1;
          const facing = (nx * ez[0] + ny * ez[1] + nz * ez[2]) / nLen;
          if (facing <= 0.02) return null;
          const shade = 0.35 + 0.65 * Math.max(0, (nx * light[0] + ny * light[1] + nz * light[2]) / nLen);
          return {
            a,
            b,
            c,
            depth: (a.depth + b.depth + c.depth) / 3,
            fill: `rgb(${Math.round(triangle.rgb[0] * shade)},${Math.round(triangle.rgb[1] * shade)},${Math.round(triangle.rgb[2] * shade)})`,
          };
        })
        .filter(Boolean)
        .sort((left, right) => left.depth - right.depth);
      for (const triangle of drawn) {
        ctx.beginPath();
        ctx.moveTo(triangle.a.x, triangle.a.y);
        ctx.lineTo(triangle.b.x, triangle.b.y);
        ctx.lineTo(triangle.c.x, triangle.c.y);
        ctx.closePath();
        ctx.fillStyle = triangle.fill;
        ctx.fill();
      }
    }, { direction: eye, focus });
    await page.screenshot({ path: path.join(shots, `${name}.png`) });
  };

  await paint('iso', [1, -1, 0.65]);
  await paint('side', [1, 0, 0.08]);
  await paint('top', [0.15, -0.2, 1]);
  await paint('under', [0.4, -0.55, -1]);
  await paint('hub', [1, 0.25, 0.2], 'hub');
  await paint('pack', [0.95, -0.7, 0.5], 'pack');
  await paint('underpack', [0.2, -0.35, -1], 'underpack');
  await paint('races', [0.35, -0.55, -1], 'races');

  await writeFile(
    path.join(shots, 'scene.json'),
    `${JSON.stringify({ kitPath, pageErrors, ...loaded }, null, 2)}\n`,
  );
  console.log(`shots → ${shots}`);
  check('no page errors', pageErrors.length === 0, pageErrors.join('\n'));
} finally {
  await browser.close();
  if (started) {
    try {
      if (started.pid) process.kill(started.pid, 'SIGTERM');
    } catch {
      started.kill();
    }
  }
}

if (failures > 0) {
  process.exit(1);
}
