/**
 * Generate research figures for docs/agentic/PRINT_KIT_AERO.md
 * from the same modified-NACA and kit-solidity math as the compilers.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const spec = JSON.parse(
  fs.readFileSync(path.join(here, "fixtures", "print-kit-tutor.spec.json"), "utf8"),
);
const outDir = path.join(here, "..", "docs", "agentic", "figures", "print-kit-aero");
fs.mkdirSync(outDir, { recursive: true });

function naca4ModifiedYtOverC(x, t, xt, leIndex) {
  const xx = Math.min(1, Math.max(0, x));
  const p = Math.min(0.42, Math.max(0.22, xt));
  const i = Math.min(9, Math.max(3, leIndex));
  const a0 = 0.2969 * (i / 6);
  const d0 = 0.002;
  const d1 = 0.234;
  const u = 1 - p;
  const rhsAft = 0.1 - d0 - d1 * u;
  const d3 = (-2 * (rhsAft + d1 * u * 0.5)) / u ** 3;
  const d2 = (-d1 - 3 * d3 * u * u) / (2 * u);
  const ypp = 2 * d2 + 6 * d3 * u;
  const s = Math.sqrt(p);
  const rhs0 = 0.1 - a0 * s;
  const rhs1 = (-0.5 * a0) / s;
  const rhs2 = ypp + 0.25 * a0 / p ** 1.5;
  const a3 = (rhs0 - p * rhs1 + 0.5 * p * p * rhs2) / p ** 3;
  const a2 = rhs2 * 0.5 - 3 * a3 * p;
  const a1 = rhs1 - p * rhs2 + 3 * a3 * p * p;
  const y20 =
    xx <= p
      ? a0 * Math.sqrt(xx) + a1 * xx + a2 * xx * xx + a3 * xx ** 3
      : d0 + d1 * (1 - xx) + d2 * (1 - xx) ** 2 + d3 * (1 - xx) ** 3;
  return Math.max(0, y20 * (t / 0.2));
}

function nacaLoop(t, xt, i, n = 97, teMinOverC = 0) {
  const xs = [];
  for (let k = 0; k < n; k++) {
    const beta = (Math.PI * k) / (n - 1);
    xs.push(0.5 * (1 - Math.cos(beta)));
  }
  const upper = [];
  const lower = [];
  for (const x of xs) {
    let yt = naca4ModifiedYtOverC(x, t, xt, i);
    if (teMinOverC > 0 && x >= 0.75) yt = Math.max(yt, teMinOverC / 2);
    if (teMinOverC > 0 && x >= 0.999) yt = teMinOverC / 2;
    upper.push([x, yt]);
    lower.push([x, -yt]);
  }
  return { xs, upper, lower, closed: [...upper, ...lower.slice().reverse().slice(1)] };
}

function esc(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;");
}

function svgDoc(w, h, body, title) {
  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" role="img" aria-label="${esc(title)}">
  <rect width="${w}" height="${h}" fill="#fbfaf6"/>
  ${body}
</svg>
`;
}

function poly(pts, attrs) {
  return `<polygon points="${pts.map(([x, y]) => `${x.toFixed(2)},${y.toFixed(2)}`).join(" ")}" ${attrs}/>`;
}

function polyline(pts, attrs) {
  return `<polyline points="${pts.map(([x, y]) => `${x.toFixed(2)},${y.toFixed(2)}`).join(" ")}" fill="none" ${attrs}/>`;
}

function text(x, y, s, extra = "") {
  return `<text x="${x}" y="${y}" font-family="Georgia, 'Times New Roman', serif" font-size="13" fill="#1f2933" ${extra}>${esc(s)}</text>`;
}

function mapX(x, x0, x1, px0, px1) {
  return px0 + ((x - x0) / (x1 - x0)) * (px1 - px0);
}

function writeFig(name, svg) {
  const dest = path.join(outDir, name);
  fs.writeFileSync(dest, svg);
  return dest;
}

// --- Fig 01: section overlay ---
{
  const foils = [
    { name: "NACA 0024-4.5/3.5  (live)", t: 0.24, xt: 0.35, i: 4.5, color: "#0f6b4c", fill: "rgba(15,107,76,0.18)", w: 2.6 },
    { name: "NACA 0021-6.0/3.0  (organic snapshot)", t: 0.21, xt: 0.3, i: 6, color: "#8a6a12", fill: "none", w: 1.6 },
    { name: "NACA 0018-6.0/3.0  (Tirandaz baseline)", t: 0.18, xt: 0.3, i: 6, color: "#6b7280", fill: "none", w: 1.4 },
  ];
  const W = 920;
  const H = 420;
  const pad = { l: 70, r: 36, t: 56, b: 70 };
  const yScale = 2.15;
  const parts = [
    text(28, 32, "Fig. 1  Modified-NACA half-thickness family (same ordinates as the kit compilers)", 'font-size="16" font-weight="600"'),
  ];
  for (const foil of foils) {
    const loop = nacaLoop(foil.t, foil.xt, foil.i);
    const pts = loop.closed.map(([x, y]) => [
      mapX(x, 0, 1, pad.l, W - pad.r),
      mapX(y, -0.14, 0.14, H - pad.b, pad.t) / 1 + (H / 2 - (H - pad.b + pad.t) / 2) * 0,
    ]);
    const mapped = loop.closed.map(([x, y]) => [
      mapX(x, 0, 1, pad.l, W - pad.r),
      mapX(y * yScale, -0.32, 0.32, H - pad.b, pad.t),
    ]);
    if (foil.fill !== "none") parts.push(poly(mapped, `fill="${foil.fill}" stroke="${foil.color}" stroke-width="${foil.w}"`));
    else parts.push(polyline(mapped.concat([mapped[0]]), `stroke="${foil.color}" stroke-width="${foil.w}"`));
  }
  const xAxisY = mapX(0, -0.32, 0.32, H - pad.b, pad.t);
  parts.push(`<line x1="${pad.l}" y1="${xAxisY}" x2="${W - pad.r}" y2="${xAxisY}" stroke="#9aa3ad" stroke-width="1"/>`);
  for (const [xv, label] of [[0, "LE"], [0.35, "xt/c = 0.35"], [0.75, "TE floor"], [1, "TE"]]) {
    const px = mapX(xv, 0, 1, pad.l, W - pad.r);
    parts.push(`<line x1="${px}" y1="${xAxisY - 4}" x2="${px}" y2="${xAxisY + 4}" stroke="#6b7280"/>`);
    parts.push(text(px - 18, H - 36, label, 'font-size="12"'));
  }
  parts.push(`<line x1="${mapX(0.35, 0, 1, pad.l, W - pad.r)}" y1="${pad.t}" x2="${mapX(0.35, 0, 1, pad.l, W - pad.r)}" y2="${H - pad.b}" stroke="#0f6b4c" stroke-dasharray="4 4" stroke-width="1"/>`);
  let ly = 64;
  for (const foil of foils) {
    parts.push(`<line x1="560" y1="${ly - 4}" x2="610" y2="${ly - 4}" stroke="${foil.color}" stroke-width="3"/>`);
    parts.push(text(618, ly, foil.name, 'font-size="12"'));
    ly += 20;
  }
  parts.push(text(pad.l, H - 14, "x/c   (cosine stations).  y/c shown 2.15× so thickness is readable.  Live TE blunt is applied only on the printed loop.", 'font-size="11" fill="#52606d"'));
  writeFig("fig-01-section-family.svg", svgDoc(W, H, parts.join("\n  "), "Modified NACA section family"));
}

// --- Fig 02: yt(x) with coefficients ---
{
  const W = 920;
  const H = 400;
  const pad = { l: 70, r: 30, t: 50, b: 56 };
  const live = { t: 0.24, xt: 0.35, i: 4.5 };
  const xs = [];
  for (let k = 0; k <= 200; k++) xs.push(k / 200);
  const ys = xs.map((x) => naca4ModifiedYtOverC(x, live.t, live.xt, live.i));
  const yMax = 0.14;
  const pts = xs.map((x, i) => [
    mapX(x, 0, 1, pad.l, W - pad.r),
    mapX(ys[i], 0, yMax, H - pad.b, pad.t),
  ]);
  const parts = [
    text(28, 30, "Fig. 2  Half-thickness yt/c of the live section  (Ladson / NASA TM 4741, t/0.20 scale)", 'font-size="16" font-weight="600"'),
    polyline(pts, 'stroke="#0f6b4c" stroke-width="2.4"'),
    `<line x1="${pad.l}" y1="${H - pad.b}" x2="${W - pad.r}" y2="${H - pad.b}" stroke="#9aa3ad"/>`,
    `<line x1="${pad.l}" y1="${pad.t}" x2="${pad.l}" y2="${H - pad.b}" stroke="#9aa3ad"/>`,
  ];
  const xtPx = mapX(0.35, 0, 1, pad.l, W - pad.r);
  const ytMax = naca4ModifiedYtOverC(0.35, 0.24, 0.35, 4.5);
  const ytPx = mapX(ytMax, 0, yMax, H - pad.b, pad.t);
  parts.push(`<line x1="${xtPx}" y1="${H - pad.b}" x2="${xtPx}" y2="${ytPx}" stroke="#c05621" stroke-dasharray="5 4"/>`);
  parts.push(`<circle cx="${xtPx}" cy="${ytPx}" r="4" fill="#c05621"/>`);
  parts.push(text(xtPx + 8, ytPx - 8, `yt/c(xt) = ${ytMax.toFixed(4)}   (t/c = 2 yt/c = ${(2 * ytMax).toFixed(3)})`, 'font-size="12" fill="#c05621"'));
  for (const xv of [0, 0.25, 0.5, 0.75, 1]) {
    parts.push(text(mapX(xv, 0, 1, pad.l, W - pad.r) - 8, H - 22, String(xv), 'font-size="12"'));
  }
  parts.push(text(pad.l - 52, pad.t + 8, "yt/c", 'font-size="12"'));
  parts.push(text(W / 2 - 20, H - 8, "x/c", 'font-size="12"'));
  parts.push(text(pad.l + 12, pad.t + 28, "fore: a0√x + a1 x + a2 x² + a3 x³", 'font-size="12" fill="#52606d"'));
  parts.push(text(pad.l + 420, pad.t + 28, "aft: d0 + d1(1−x) + d2(1−x)² + d3(1−x)³", 'font-size="12" fill="#52606d"'));
  writeFig("fig-02-half-thickness.svg", svgDoc(W, H, parts.join("\n  "), "Half-thickness of live section"));
}

// --- Fig 03: geometric alpha vs azimuth ---
{
  const W = 920;
  const H = 420;
  const pad = { l: 70, r: 36, t: 50, b: 56 };
  const lambdas = [
    { l: 2.0, color: "#9b1c1c" },
    { l: 2.5, color: "#0f6b4c", w: 2.6 },
    { l: 3.0, color: "#2b6cb0" },
    { l: 4.5, color: "#6b7280" },
  ];
  const parts = [
    text(28, 30, "Fig. 3  Geometric angle of attack  α(θ) = atan2(sin θ,  λ + cos θ)   (no induction)", 'font-size="16" font-weight="600"'),
    `<line x1="${pad.l}" y1="${mapX(0, -50, 50, H - pad.b, pad.t)}" x2="${W - pad.r}" y2="${mapX(0, -50, 50, H - pad.b, pad.t)}" stroke="#d2d6dc"/>`,
    `<line x1="${pad.l}" y1="${pad.t}" x2="${pad.l}" y2="${H - pad.b}" stroke="#9aa3ad"/>`,
    `<line x1="${pad.l}" y1="${H - pad.b}" x2="${W - pad.r}" y2="${H - pad.b}" stroke="#9aa3ad"/>`,
  ];
  for (const row of lambdas) {
    const pts = [];
    for (let k = 0; k <= 360; k++) {
      const th = (k * Math.PI) / 180;
      const a = (Math.atan2(Math.sin(th), row.l + Math.cos(th)) * 180) / Math.PI;
      pts.push([mapX(k, 0, 360, pad.l, W - pad.r), mapX(a, -50, 50, H - pad.b, pad.t)]);
    }
    parts.push(polyline(pts, `stroke="${row.color}" stroke-width="${row.w || 1.6}"`));
  }
  let ly = 64;
  for (const row of lambdas) {
    parts.push(`<line x1="720" y1="${ly - 4}" x2="760" y2="${ly - 4}" stroke="${row.color}" stroke-width="3"/>`);
    parts.push(text(768, ly, `λ = ${row.l.toFixed(1)}`, 'font-size="12"'));
    ly += 18;
  }
  parts.push(text(pad.l - 48, pad.t + 10, "α [°]", 'font-size="12"'));
  parts.push(text(W / 2 - 40, H - 12, "azimuth θ [°]   (θ = 0 downwind)", 'font-size="12"'));
  parts.push(text(pad.l + 8, H - 70, "At λ = 2.5 the blade sees |α| ≳ 20° — dynamic stall. That is why the live foil is the thick aft 0024, not a high-λ 0010.", 'font-size="11" fill="#52606d"'));
  writeFig("fig-03-alpha-azimuth.svg", svgDoc(W, H, parts.join("\n  "), "Geometric angle of attack vs azimuth"));
}

// --- Fig 04: relative speed ---
{
  const W = 920;
  const H = 380;
  const pad = { l: 70, r: 36, t: 50, b: 56 };
  const parts = [
    text(28, 30, "Fig. 4  Relative speed  |Vrel|/V∞ = √(1 + 2λ cos θ + λ²)   (no induction)", 'font-size="16" font-weight="600"'),
    `<line x1="${pad.l}" y1="${H - pad.b}" x2="${W - pad.r}" y2="${H - pad.b}" stroke="#9aa3ad"/>`,
    `<line x1="${pad.l}" y1="${pad.t}" x2="${pad.l}" y2="${H - pad.b}" stroke="#9aa3ad"/>`,
  ];
  for (const row of [
    { l: 2.5, color: "#0f6b4c", w: 2.6 },
    { l: 3.0, color: "#2b6cb0" },
    { l: 4.5, color: "#6b7280" },
  ]) {
    const pts = [];
    for (let k = 0; k <= 360; k++) {
      const th = (k * Math.PI) / 180;
      const vr = Math.sqrt(1 + 2 * row.l * Math.cos(th) + row.l * row.l);
      pts.push([mapX(k, 0, 360, pad.l, W - pad.r), mapX(vr, 1, 6, H - pad.b, pad.t)]);
    }
    parts.push(polyline(pts, `stroke="${row.color}" stroke-width="${row.w || 1.6}"`));
  }
  parts.push(text(pad.l - 56, pad.t + 10, "|Vrel|/V∞", 'font-size="12"'));
  parts.push(text(W / 2 - 20, H - 12, "azimuth θ [°]", 'font-size="12"'));
  writeFig("fig-04-vrel.svg", svgDoc(W, H, parts.join("\n  "), "Relative speed vs azimuth"));
}

// --- Fig 05: helix coverage ---
{
  const W = 920;
  const H = 420;
  const pad = { l: 80, r: 40, t: 56, b: 50 };
  const parts = [
    text(28, 30, "Fig. 5  Helix unwrap — three blades, 60° from a 30° root  (even-machine golden)", 'font-size="16" font-weight="600"'),
    `<line x1="${pad.l}" y1="${H - pad.b}" x2="${W - pad.r}" y2="${H - pad.b}" stroke="#9aa3ad"/>`,
    `<line x1="${pad.l}" y1="${pad.t}" x2="${pad.l}" y2="${H - pad.b}" stroke="#9aa3ad"/>`,
  ];
  const colors = ["#0f6b4c", "#2b6cb0", "#c05621"];
  for (let b = 0; b < 3; b++) {
    const root = 30 + 120 * b;
    const tip = root + 60;
    const pts = [
      [mapX(root, 0, 360, pad.l, W - pad.r), mapX(0, 0, 1, H - pad.b, pad.t)],
      [mapX(tip, 0, 360, pad.l, W - pad.r), mapX(1, 0, 1, H - pad.b, pad.t)],
    ];
    parts.push(polyline(pts, `stroke="${colors[b]}" stroke-width="4"`));
    parts.push(text(pts[0][0] + 6, pts[0][1] - 8, `blade ${b + 1}  θroot=${root}°`, `font-size="12" fill="${colors[b]}"`));
  }
  const mid = 60;
  const mx = mapX(mid, 0, 360, pad.l, W - pad.r);
  parts.push(`<line x1="${mx}" y1="${pad.t}" x2="${mx}" y2="${H - pad.b}" stroke="#0f6b4c" stroke-dasharray="4 4"/>`);
  parts.push(text(mx + 6, pad.t + 16, "mid-span of blade 1 at 60°", 'font-size="12" fill="#0f6b4c"'));
  parts.push(text(pad.l - 64, pad.t + 8, "span η", 'font-size="12"'));
  parts.push(text(W / 2 - 50, H - 14, "azimuth θ [°]     θ_mid = θ_root + ψ/2 = 30 + 30 = 60", 'font-size="12"'));
  writeFig("fig-05-helix-unwrap.svg", svgDoc(W, H, parts.join("\n  "), "Helix unwrap of three blades"));
}

// --- Fig 06: plan / solidity ---
{
  const W = 720;
  const H = 720;
  const cx = 360;
  const cy = 378;
  const R = spec.wing_radius;
  const c = spec.wing_chord_root;
  const tipR = R + spec.wing_chord_tip * 0.15;
  const scale = 2.55;
  const px = (mm) => mm * scale;
  const parts = [
    text(24, 32, "Fig. 6  Plan: mid-chord on the cylinder, chord tangent  (scale 1.0 mm)", 'font-size="16" font-weight="600"'),
    `<circle cx="${cx}" cy="${cy}" r="${px(R)}" fill="none" stroke="#0f6b4c" stroke-width="1.6"/>`,
    `<circle cx="${cx}" cy="${cy}" r="${px(tipR)}" fill="none" stroke="#9aa3ad" stroke-dasharray="5 4"/>`,
    `<circle cx="${cx}" cy="${cy}" r="3" fill="#1f2933"/>`,
  ];
  for (let b = 0; b < 3; b++) {
    const th = ((30 + 120 * b) * Math.PI) / 180;
    const mx = cx + px(R) * Math.cos(th);
    const my = cy + px(R) * Math.sin(th);
    const tx = -Math.sin(th);
    const ty = Math.cos(th);
    const half = px(c) / 2;
    const loop = nacaLoop(0.24, 0.35, 4.5, 41);
    const foil = loop.closed.map(([x, y]) => {
      const xc = (x - 0.5) * px(c);
      const yc = y * px(c);
      return [mx + tx * xc - Math.cos(th) * yc, my + ty * xc - Math.sin(th) * yc];
    });
    parts.push(poly(foil, 'fill="rgba(15,107,76,0.22)" stroke="#0f6b4c" stroke-width="1.4"'));
    parts.push(`<line x1="${cx}" y1="${cy}" x2="${mx}" y2="${my}" stroke="#d2d6dc"/>`);
  }
  const sigmaCyl = (spec.wing_count * spec.wing_chord_root) / (Math.PI * 2 * spec.wing_radius);
  const rotorD = 2 * tipR;
  const sigmaKit = (spec.wing_count * spec.wing_chord_root) / (Math.PI * rotorD);
  parts.push(text(24, 58, `R = ${R} mm   c_root = ${c} mm   N = 3`, 'font-size="13"'));
  parts.push(text(24, 78, `σ_cyl = Nc / (π · 2R) = ${sigmaCyl.toFixed(3)}     σ_kit = Nc / (π D_tip) = ${sigmaKit.toFixed(3)}`, 'font-size="13"'));
  parts.push(text(24, H - 24, "Dashed circle is blade_tip_r = R + 0.15 c_tip — the kit solidity uses that D.", 'font-size="12" fill="#52606d"'));
  writeFig("fig-06-plan-solidity.svg", svgDoc(W, H, parts.join("\n  "), "Plan view and solidity"));
}

// --- Fig 07: lambda-optimum map ---
{
  const rows = [
    [2.5, "NACA 0024-4.5/3.5", 0.24, 0.35, 4.5, "LIVE KIT"],
    [3.0, "NACA 0018-4.5/2.75", 0.18, 0.275, 4.5, ""],
    [3.5, "NACA 0015-4.5/2.50", 0.15, 0.25, 4.5, ""],
    [4.5, "NACA 0012-4.5/2.25", 0.12, 0.225, 4.5, ""],
    [5.5, "NACA 0010-6.0/2.25", 0.1, 0.225, 6.0, ""],
  ];
  const W = 920;
  const H = 360;
  const parts = [
    text(28, 32, "Fig. 7  Published optimum symmetric section vs λ  (Tirandaz, Rezaeiha & Micallef, WES 2023)", 'font-size="16" font-weight="600"'),
    `<rect x="28" y="52" width="864" height="36" fill="#e6f4ee"/>`,
  ];
  const heads = ["λ", "optimum section", "t/c", "xt/c", "I", ""];
  const xs = [40, 130, 430, 540, 650, 740];
  heads.forEach((h, i) => parts.push(text(xs[i], 76, h, 'font-size="13" font-weight="600"')));
  rows.forEach((row, r) => {
    const y = 118 + r * 40;
    if (row[5]) parts.push(`<rect x="28" y="${y - 24}" width="864" height="40" fill="#e6f4ee"/>`);
    row.forEach((cell, i) => parts.push(text(xs[i], y, String(cell), `font-size="14"${row[5] && i === 5 ? ' fill="#0f6b4c" font-weight="600"' : ""}`)));
  });
  writeFig("fig-07-lambda-map.svg", svgDoc(W, H, parts.join("\n  "), "Optimum section versus tip-speed ratio"));
}

// --- numbers sidecar for the markdown ---
const tipR = spec.wing_radius + spec.wing_chord_tip * 0.15;
const rotorD = 2 * tipR;
const sigmaKit = (spec.wing_count * spec.wing_chord_root) / (Math.PI * rotorD);
const sigmaCyl = (spec.wing_count * spec.wing_chord_root) / (Math.PI * 2 * spec.wing_radius);
const yt = naca4ModifiedYtOverC(0.35, 0.24, 0.35, 4.5);
const rLe = 1.1019 * ((4.5 / 6) * 0.24) ** 2;
const numbers = {
  generated: new Date().toISOString(),
  spec_id: spec.id,
  airfoil: spec.airfoil,
  t_c: spec.airfoil_t_c,
  xt_c: spec.airfoil_xt_c,
  le_index: spec.airfoil_le_index,
  yt_over_c_at_xt: Number(yt.toFixed(6)),
  t_over_c_from_2yt: Number((2 * yt).toFixed(6)),
  r_le_over_c: Number(rLe.toFixed(6)),
  scale_1: {
    R_mm: spec.wing_radius,
    H_mm: spec.wing_h,
    c_root_mm: spec.wing_chord_root,
    c_tip_mm: spec.wing_chord_tip,
    blade_tip_r_mm: Number(tipR.toFixed(3)),
    rotor_d_mm: Number(rotorD.toFixed(3)),
    sigma_kit: Number(sigmaKit.toFixed(6)),
    sigma_cylinder: Number(sigmaCyl.toFixed(6)),
    helix_deg: spec.helix_deg,
    wing_offset_deg: spec.wing_offset_deg,
    mid_azimuth_deg: spec.wing_offset_deg + spec.helix_deg * 0.5,
  },
  scale_exam: {
    scale: spec.scale,
    R_mm: spec.wing_radius * spec.scale,
    H_mm: spec.wing_h * spec.scale,
    c_root_mm: spec.wing_chord_root * spec.scale,
    c_tip_mm: spec.wing_chord_tip * spec.scale,
  },
};
fs.writeFileSync(path.join(outDir, "numbers.json"), JSON.stringify(numbers, null, 2));
console.log(JSON.stringify(numbers, null, 2));
console.log("wrote", outDir);
