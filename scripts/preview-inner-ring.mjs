#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { unzipSync } from "fflate";
import { deflateSync } from "node:zlib";
import os from "node:os";
import path from "node:path";

const src = path.join(os.homedir(), "Documents", "noBS-CAD", "Print-Kit-Inner-Ring", "01-inner-ring.3mf");
const dest = path.join(os.homedir(), "Documents", "noBS-CAD", "Print-Kit-Inner-Ring", "01-inner-ring-preview.png");

const zip = unzipSync(readFileSync(src));
const xml = new TextDecoder().decode(zip["3D/3dmodel.model"]);
const verts = [];
for (const m of xml.matchAll(/<vertex x="([^"]+)" y="([^"]+)" z="([^"]+)"/g)) {
  verts.push([Number(m[1]), Number(m[2]), Number(m[3])]);
}
const tris = [];
for (const m of xml.matchAll(/<triangle v1="([^"]+)" v2="([^"]+)" v3="([^"]+)"/g)) {
  tris.push([Number(m[1]), Number(m[2]), Number(m[3])]);
}

let minR = Infinity;
let maxR = 0;
let minZ = Infinity;
let maxZ = -Infinity;
let volume = 0;
for (const [x, y, z] of verts) {
  const r = Math.hypot(x, y);
  minR = Math.min(minR, r);
  maxR = Math.max(maxR, r);
  minZ = Math.min(minZ, z);
  maxZ = Math.max(maxZ, z);
}
for (const [i, j, k] of tris) {
  const a = verts[i];
  const b = verts[j];
  const c = verts[k];
  volume +=
    (a[0] * (b[1] * c[2] - b[2] * c[1]) -
      a[1] * (b[0] * c[2] - b[2] * c[0]) +
      a[2] * (b[0] * c[1] - b[1] * c[0])) /
    6;
}
volume = Math.abs(volume);

function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c >>> 0;
}
function chunk(type, data) {
  const out = Buffer.alloc(12 + data.length);
  out.writeUInt32BE(data.length, 0);
  out.write(type, 4);
  data.copy(out, 8);
  const crc = crc32(out.subarray(4, 8 + data.length));
  out.writeUInt32BE(crc, 8 + data.length);
  return out;
}
function writePng(width, height, rgb) {
  const raw = Buffer.alloc((width * 3 + 1) * height);
  for (let y = 0; y < height; y++) {
    raw[y * (width * 3 + 1)] = 0;
    rgb.copy(raw, y * (width * 3 + 1) + 1, y * width * 3, (y + 1) * width * 3);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw, { level: 6 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function render(eye, width, height) {
  const rgb = Buffer.alloc(width * height * 3);
  const zbuf = new Float64Array(width * height);
  zbuf.fill(-Infinity);
  const bg = [213, 219, 226];
  for (let i = 0; i < width * height; i++) {
    rgb[i * 3] = bg[0];
    rgb[i * 3 + 1] = bg[1];
    rgb[i * 3 + 2] = bg[2];
  }
  const elen = Math.hypot(eye[0], eye[1], eye[2]);
  const ez = [eye[0] / elen, eye[1] / elen, eye[2] / elen];
  const up0 = Math.abs(ez[2]) > 0.9 ? [0, 1, 0] : [0, 0, 1];
  const ex = [
    up0[1] * ez[2] - up0[2] * ez[1],
    up0[2] * ez[0] - up0[0] * ez[2],
    up0[0] * ez[1] - up0[1] * ez[0],
  ];
  const xl = Math.hypot(ex[0], ex[1], ex[2]);
  ex[0] /= xl;
  ex[1] /= xl;
  ex[2] /= xl;
  const ey = [
    ez[1] * ex[2] - ez[2] * ex[1],
    ez[2] * ex[0] - ez[0] * ex[2],
    ez[0] * ex[1] - ez[1] * ex[0],
  ];
  const light = [0.35, 0.55, 0.76];
  const ll = Math.hypot(light[0], light[1], light[2]);
  light[0] /= ll;
  light[1] /= ll;
  light[2] /= ll;
  const project = (p) => [
    p[0] * ex[0] + p[1] * ex[1] + p[2] * ex[2],
    p[0] * ey[0] + p[1] * ey[1] + p[2] * ey[2],
    p[0] * ez[0] + p[1] * ez[1] + p[2] * ez[2],
  ];
  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;
  const proj = verts.map((v) => {
    const p = project(v);
    minX = Math.min(minX, p[0]);
    maxX = Math.max(maxX, p[0]);
    minY = Math.min(minY, p[1]);
    maxY = Math.max(maxY, p[1]);
    return p;
  });
  const span = Math.max(maxX - minX, maxY - minY) * 1.12;
  const cx = (minX + maxX) / 2;
  const cy = (minY + maxY) / 2;
  const sx = width / span;
  const sy = height / span;
  const s = Math.min(sx, sy);
  const ox = width / 2 - cx * s;
  const oy = height / 2 + cy * s;
  const base = [240, 120, 40];
  for (const [i, j, k] of tris) {
    const a = proj[i];
    const b = proj[j];
    const c = proj[k];
    const abx = b[0] - a[0];
    const aby = b[1] - a[1];
    const abz = b[2] - a[2];
    const acx = c[0] - a[0];
    const acy = c[1] - a[1];
    const acz = c[2] - a[2];
    const nx = aby * acz - abz * acy;
    const ny = abz * acx - abx * acz;
    const nz = abx * acy - aby * acx;
    const nl = Math.hypot(nx, ny, nz) || 1;
    const ndot = (nx * light[0] + ny * light[1] + nz * light[2]) / nl;
    const shade = 0.28 + 0.72 * Math.max(0, ndot);
    const col = [
      Math.min(255, Math.round(base[0] * shade)),
      Math.min(255, Math.round(base[1] * shade)),
      Math.min(255, Math.round(base[2] * shade)),
    ];
    const ax = a[0] * s + ox;
    const ay = -a[1] * s + oy;
    const bx = b[0] * s + ox;
    const by = -b[1] * s + oy;
    const cxp = c[0] * s + ox;
    const cyp = -c[1] * s + oy;
    const z = (a[2] + b[2] + c[2]) / 3;
    const minXi = Math.max(0, Math.floor(Math.min(ax, bx, cxp)));
    const maxXi = Math.min(width - 1, Math.ceil(Math.max(ax, bx, cxp)));
    const minYi = Math.max(0, Math.floor(Math.min(ay, by, cyp)));
    const maxYi = Math.min(height - 1, Math.ceil(Math.max(ay, by, cyp)));
    const area = (bx - ax) * (cyp - ay) - (by - ay) * (cxp - ax);
    if (Math.abs(area) < 1e-6) continue;
    for (let y = minYi; y <= maxYi; y++) {
      for (let x = minXi; x <= maxXi; x++) {
        const w0 = ((bx - ax) * (y - ay) - (by - ay) * (x - ax)) / area;
        const w1 = ((cxp - bx) * (y - by) - (cyp - by) * (x - bx)) / area;
        const w2 = ((ax - cxp) * (y - cyp) - (ay - cyp) * (x - cxp)) / area;
        if (w0 < -0.01 || w1 < -0.01 || w2 < -0.01) continue;
        const idx = y * width + x;
        if (z >= zbuf[idx]) {
          zbuf[idx] = z;
          rgb[idx * 3] = col[0];
          rgb[idx * 3 + 1] = col[1];
          rgb[idx * 3 + 2] = col[2];
        }
      }
    }
  }
  return rgb;
}

const W = 720;
const H = 720;
const top = render([0, 0, 1], W, H);
const iso = render([1.1, 0.85, 0.75], W, H);
const out = Buffer.alloc(W * 2 * H * 3);
for (let y = 0; y < H; y++) {
  top.copy(out, y * W * 2 * 3, y * W * 3, (y + 1) * W * 3);
  iso.copy(out, y * W * 2 * 3 + W * 3, y * W * 3, (y + 1) * W * 3);
}
writeFileSync(dest, writePng(W * 2, H, out));
console.log(
  `verts=${verts.length} tris=${tris.length} r=${minR.toFixed(2)}..${maxR.toFixed(2)} z=${minZ.toFixed(2)}..${maxZ.toFixed(2)} vol=${(volume / 1000).toFixed(2)} cm3`,
);
console.log(dest);
