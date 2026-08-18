/**
 * Figures for docs/agentic/PRINT_KIT_BEARING.md
 * Exam-scale (0.4) numbers from the same helpers as the tutor.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const spec = JSON.parse(
  fs.readFileSync(path.join(here, "fixtures", "print-kit-tutor.spec.json"), "utf8"),
);
const outDir = path.join(here, "..", "docs", "agentic", "figures", "print-kit-bearing");
fs.mkdirSync(outDir, { recursive: true });

const scale = spec.scale;
const nozzle = spec.nozzle_mm;
const wall = Math.max(4 * scale, nozzle * 4);
const rollerD = Math.max(spec.roller_d * scale, spec.roller_min_d);
const rollerL = Math.max(spec.roller_len * scale, 8);
const packH = rollerD;
const fenceH = Math.max(packH * 0.62, wall * 2);
const topLoad = nozzle * 2;
const slotD = rollerD + spec.fit_running_mm + topLoad;
const pocketL = rollerL + spec.fit_running_mm;
const zMid = packH / 2;
const troughDepth = slotD / 2 - zMid;
const clipId = 10.68;
const journal = 12.0;
const hoopStrain = (journal - clipId) / clipId;

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
function text(x, y, s, extra = "") {
  return `<text x="${x}" y="${y}" font-family="Georgia, 'Times New Roman', serif" font-size="13" fill="#1f2933" ${extra}>${esc(s)}</text>`;
}

function write(name, svg) {
  fs.writeFileSync(path.join(outDir, name), svg);
}

const numbers = {
  generated: new Date().toISOString(),
  exam: {
    roller_d: rollerD,
    roller_len: rollerL,
    pack_h: packH,
    fence_h: Number(fenceH.toFixed(3)),
    slot_cut_d: slotD,
    pocket_len: Number(pocketL.toFixed(3)),
    z_mid_above_race: zMid,
    trough_into_race: Number(troughDepth.toFixed(3)),
    protrusion_above_fence: Number((packH - fenceH).toFixed(3)),
    hoop_strain: Number(hoopStrain.toFixed(4)),
  },
};
fs.writeFileSync(path.join(outDir, "numbers.json"), JSON.stringify(numbers, null, 2));

// Fig B1 — current stack cross-section (why it slides)
{
  const W = 920;
  const H = 420;
  const s = 18;
  const ox = 80;
  const oy = 300;
  const raceH = 3.2 * s;
  const fence = fenceH * s;
  const r = (slotD / 2) * s;
  const rr = (rollerD / 2) * s;
  const cx = ox + 220;
  const cy = oy - zMid * s;
  const parts = [
    text(24, 28, "Fig. B1  As-printed pack (exam 0.4) — the slot is a cylinder cut, so the bottom is a trough", 'font-size="16" font-weight="600"'),
    `<rect x="${ox}" y="${oy}" width="520" height="${raceH}" fill="#f4d7b0" stroke="#8a5a1a"/>`,
    `<rect x="${ox + 40}" y="${oy - fence}" width="80" height="${fence}" fill="#f4d7b0" stroke="#8a5a1a"/>`,
    `<rect x="${ox + 400}" y="${oy - fence}" width="80" height="${fence}" fill="#f4d7b0" stroke="#8a5a1a"/>`,
    `<path d="M ${cx - r} ${oy} A ${r} ${r} 0 0 1 ${cx + r} ${oy} L ${cx + r} ${oy} L ${cx - r} ${oy} Z" fill="#fbfaf6" stroke="#8a5a1a"/>`,
    `<circle cx="${cx}" cy="${cy}" r="${rr}" fill="rgba(15,107,76,0.25)" stroke="#0f6b4c" stroke-width="2"/>`,
    `<rect x="${ox + 60}" y="${cy - rr - 18}" width="400" height="16" fill="rgba(47,133,90,0.35)" stroke="#2f855a"/>`,
    text(ox + 70, cy - rr - 24, "plate slides here (rollers locked)", 'font-size="12" fill="#9b1c1c"'),
    text(ox + 540, oy + 20, "race ring (flat in CAD)", 'font-size="12"'),
    text(ox + 540, oy - fence / 2, `fence h=${fenceH.toFixed(2)}`, 'font-size="12"'),
    text(cx - 40, cy + 6, "Ø8 roller", 'font-size="12" fill="#0f6b4c"'),
    text(24, 56, `Slot cut Ø${slotD.toFixed(1)} centered at pack mid. Bottom of hole is ${troughDepth.toFixed(2)} mm into the race → conforming cradle, not a flat land.`, 'font-size="12" fill="#52606d"'),
    text(24, H - 18, "Opening is the same cylinder through the fence. Sides wrap the OD. That is a journal around the roller.", 'font-size="12" fill="#52606d"'),
  ];
  write("fig-b1-current-trough.svg", svgDoc(W, H, parts.join("\n  "), "Current cylindrical slot trough"));
}

// Fig B2 — proposed stack
{
  const W = 920;
  const H = 420;
  const s = 18;
  const ox = 80;
  const oy = 300;
  const raceH = 3.2 * s;
  const fence = fenceH * s;
  const rr = (rollerD / 2) * s;
  const cx = ox + 220;
  const cy = oy - rr;
  const funnel = 14;
  const parts = [
    text(24, 28, "Fig. B2  Required stack — flat race, funnel opening, crowned roller, sides do not wrap", 'font-size="16" font-weight="600"'),
    `<rect x="${ox}" y="${oy}" width="520" height="${raceH}" fill="#f4d7b0" stroke="#8a5a1a"/>`,
    `<rect x="${ox}" y="${oy}" width="520" height="6" fill="#c05621"/>`,
    `<path d="M ${ox + 40} ${oy - fence} L ${ox + 40 + funnel} ${oy} L ${ox + 40} ${oy} Z" fill="#f4d7b0" stroke="#8a5a1a"/>`,
    `<path d="M ${ox + 480} ${oy - fence} L ${ox + 480 - funnel} ${oy} L ${ox + 480} ${oy} Z" fill="#f4d7b0" stroke="#8a5a1a"/>`,
    `<ellipse cx="${cx}" cy="${cy}" rx="${rr}" ry="${rr * 0.92}" fill="rgba(15,107,76,0.28)" stroke="#0f6b4c" stroke-width="2"/>`,
    `<rect x="${ox + 60}" y="${cy - rr * 0.92}" width="400" height="10" fill="rgba(47,133,90,0.4)" stroke="#2f855a"/>`,
    text(ox + 70, cy - rr * 0.92 - 10, "plate on the crown (rolling)", 'font-size="12" fill="#0f6b4c"'),
    text(ox + 540, oy + 8, "flat race land + 0.8 mm chamfer", 'font-size="12"'),
    text(ox + 540, oy - fence + 12, "funnel opening 20–30°", 'font-size="12"'),
    text(24, 56, "Cut a downward U-window through the fence only. Stop at the race face. Circumferential clearance ≥ 0.8 mm/side. Keepers touch ends, not the generator.", 'font-size="12" fill="#52606d"'),
    text(24, H - 18, "Small barrel crown (end drop 0.25–0.40 mm). Not a cone unless both races are also cones that share an apex.", 'font-size="12" fill="#52606d"'),
  ];
  write("fig-b2-required-stack.svg", svgDoc(W, H, parts.join("\n  "), "Required flat race and funnel"));
}

// Fig B3 — apex rule vs cone-on-flat
{
  const W = 920;
  const H = 360;
  const parts = [
    text(24, 28, "Fig. B3  A taper only works if the races share the roller apex  (Timken / ISO 355)", 'font-size="16" font-weight="600"'),
    `<line x1="80" y1="80" x2="80" y2="320" stroke="#1f2933" stroke-width="2"/>`,
    `<line x1="80" y1="80" x2="400" y2="140" stroke="#0f6b4c" stroke-width="2"/>`,
    `<line x1="80" y1="80" x2="400" y2="220" stroke="#0f6b4c" stroke-width="2"/>`,
    `<polygon points="250,128 360,155 360,205 250,192" fill="rgba(15,107,76,0.25)" stroke="#0f6b4c"/>`,
    `<circle cx="80" cy="80" r="4" fill="#c05621"/>`,
    text(90, 74, "common apex on the bearing axis", 'font-size="12" fill="#c05621"'),
    text(250, 250, "TRUE TAPER — pure rolling", 'font-size="13" fill="#0f6b4c"'),
    `<rect x="520" y="140" width="340" height="14" fill="#d2d6dc" stroke="#6b7280"/>`,
    `<rect x="520" y="230" width="340" height="14" fill="#d2d6dc" stroke="#6b7280"/>`,
    `<polygon points="560,154 820,154 800,230 580,230" fill="rgba(155,28,28,0.2)" stroke="#9b1c1c"/>`,
    text(560, 280, "CONE ON TWO FLATS — scrub", 'font-size="13" fill="#9b1c1c"'),
    text(24, 340, "ωr changes along the cone; flats force one speed. That is more sliding, not less.", 'font-size="12" fill="#52606d"'),
  ];
  write("fig-b3-apex-rule.svg", svgDoc(W, H, parts.join("\n  "), "Apex rule versus cone on flats"));
}

// Fig B4 — crown drop
{
  const W = 920;
  const H = 320;
  const L = 700;
  const x0 = 110;
  const y0 = 200;
  const pts = [];
  for (let i = 0; i <= 80; i++) {
    const t = i / 80;
    const x = (t - 0.5) * 2;
    const drop = 28 * (x * x);
    pts.push(`${x0 + t * L},${y0 + drop}`);
  }
  const parts = [
    text(24, 28, "Fig. B4  Printable stand-in for Lundberg crown — circular barrel, end drop 0.25–0.40 mm", 'font-size="16" font-weight="600"'),
    `<polyline points="${pts.join(" ")}" fill="none" stroke="#0f6b4c" stroke-width="3"/>`,
    `<line x1="${x0}" y1="${y0}" x2="${x0 + L}" y2="${y0}" stroke="#9aa3ad" stroke-dasharray="5 4"/>`,
    text(x0, y0 - 12, "mid Ø (pack height)", 'font-size="12"'),
    text(x0 + L - 80, y0 + 48, "end drop δ", 'font-size="12" fill="#c05621"'),
    text(24, 56, "ISO/TS 16281 logarithmic crown: z(x) = (Q/(π L E')) ln(1/(1−(2x/L)²)). FDM cannot hold that. A circular barrel with chamfered ends is the printable proxy.", 'font-size="12" fill="#52606d"'),
    text(24, 300, "Chamfer both ends 0.4–0.6 mm so keepers never see a sharp generator.", 'font-size="12" fill="#52606d"'),
  ];
  write("fig-b4-crown.svg", svgDoc(W, H, parts.join("\n  "), "Barrel crown"));
}

console.log(JSON.stringify(numbers, null, 2));
console.log("wrote", outDir);
