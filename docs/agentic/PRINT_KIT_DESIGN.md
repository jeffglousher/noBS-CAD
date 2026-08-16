# Print-kit tutor — design contract (2026)

Adversarial contract for benchmark #1. The recipe is `prompts/get model_print_kit`
(`mcp-server/src/prompts/model_print_kit.md`). Numbers live in
`scripts/fixtures/print-kit-tutor.spec.json`. Fits and closed faults:
[PRINT_KIT_GDT.md](PRINT_KIT_GDT.md).

A kit that exports 3MF and cannot make directionless torque is a **fail**.
A kit whose frame out-girths the rotor, or whose blades are a straight
fence, is also a **fail**.

## Grouped expected design

| Group | Number | Why |
|-------|--------|-----|
| Section | NACA 0021, TE ≥ 0.8 mm | 2026 thick-symmetric band; printable |
| Rotor | c=16, R=24, span=48, N=3, 60° helix | σ≈0.32; AR=3; always some blade working |
| Bay | Root 30°, helix +60° | Stays between Ø5 posts on R32 |
| Stand | Y-frame (hub+ribs+pads), Ø5 posts | Envelope/rotor ≤ 1.55; no cookie plate |
| Fits | +0.40 mm, two-land sleeve L/D 1.0 | No 608, no screws; do not tighten |

Straight H-rotor is the wrong default at this Re: torque ripple and
dead azimuths. Helix is required. Mid-chord stays on the cylinder;
chord stays tangent. Do not twist the section in place.

## Why this page exists

The exam used to accept geometry that was not a turbine: leftover helical
C-buckets, concentric hoop sectors, then a flat plate. The grader said
READY TO PRINT. This page is the research bar the prompt now enforces.

## Directionless VAWT

A vertical-axis machine does not yaw. Wind from any azimuth must produce
the same family of torque. That requires:

- Odd blade count (3), equal spacing
- **Symmetric** airfoil (reversing angle of attack each rev)
- Chord approximately tangential, identical blades
- No preferred wind direction, no one-sided vane

Savonius-only is directionless and inefficient (drag). HAWT sections are
the wrong physics. H-Darrieus with a thick symmetric section is the
required architecture on the printed Y-frame stand.

Optional inner drag starter (Savonius cup, or the 2026 adaptive
Darrieus–Savonius flap) must not replace the airfoil and must assemble
with no extra hardware.

## 2026 section research (cite what you build)

| Source | Year | What it says for this kit |
|--------|------|---------------------------|
| Tirandaz / Rezaeiha line, *Energies* **19**(7) 1615 | 2026 | Coupled t/c, xt/c, LE radius. Favorable low-TSR band **t/c 21–24%**, **xt/c 27.5–35%**, LE index **I ≈ 4.5**. Best in that study: **NACA 0024–4.5/3.5** (+73% Cp vs NACA 0018–6.0/3.0) |
| Range-wide Darrieus CFD + MOP (Joukowsky parameterization) | 2026 | Optimized sections beat NACA 0021 across TSR (~11% mean Cp) |
| Thin vs thick + plain/Gurney/hybrid flaps, *Results in Engineering* | 2026 | Low-Re Darrieus: thick 0021 + hybrid flap can raise Cp; a flap is not a substitute for a section |
| Gu, MacDonald, Tang, *Flow* 6 | 2026 | Adaptive Darrieus–Savonius flap: +65% static torque, start 8 → 6 m/s. Optional starter, not the wing |
| Hybrid helical-inner + Darrieus-outer (Taguchi) | 2025–26 | Helical inner helps start and smooths torque. Allowed only with closed airfoil stations |

**Printable required section for the exam:** NACA **0021** (t/c = 0.21)
with trailing edge blunt to **≥ 2 nozzles**. Construct 0024–4.5/3.5 if
you can. A rectangle with the same bounding box is a vane.

Desk-scale Re (c = 12 mm, 3–5 m/s, TSR ~2) is **~5×10³–10⁴**. Thick
symmetric is the correct family. NACA 0012 is not.

## Service finish

As-printed FDM skins are rough (Ra often 10–25 µm). That is Cd.

- ABS acetone **vapor** (not immersion): Ra down ~80%, L/D up ~27% on
  printed airfoils (Ing. Investig. airfoil finish study)
- PLA: sand 400→1000 on blade skins; optional filler primer (consumable,
  not hardware)
- Layer lines **parallel to span** on blades
- Journal bores as XY circles (bushing printed as a ring)
- Do not vapor-smooth a running fit and then keep the modeled +0.40

## Bushings, not bearings

At this size, prefer printed sleeves and a printed thrust land. Metal
608 / rollers are a later catalog table ([ASSEMBLY.md](ASSEMBLY.md)).

| Station | What | Why not a roller |
|---------|------|------------------|
| Thrust | 45° cup + smaller male + **Ø12** land, 0.20 float | Wide land is extra drag; ball thrust needs a race we do not have |
| Upper radial | Two-land sleeve Ø8.4/Ø14 × 8, L/D = **1.0**, relief Ø9.2 | Full-cylinder rub and L/D 0.5 were the last cookie. 608 is hidden hardware |
| Drive | Double-D +0.40 | Set screw is hardware |
| Wing | Tenon/socket +0.40 | Screw is hardware |

PLA-on-PLA is a demo spin. Say so. No glue-as-fit.

## Report

The exam writes `Print-Kit-Tutor-design.md` next to the 3MF. Required
sections: iteration log, design process, final product, plastic/material
cost. Sample: [PRINT_KIT_REPORT.md](PRINT_KIT_REPORT.md).
