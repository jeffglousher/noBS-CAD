# Print-kit tutor — design contract (2026)

Adversarial contract for benchmark #1. The recipe is `prompts/get model_print_kit`
(`mcp-server/src/prompts/model_print_kit.md`). Numbers live in
`scripts/fixtures/print-kit-tutor.spec.json`. Fits and closed faults:
[PRINT_KIT_GDT.md](PRINT_KIT_GDT.md).

Start from a **blank document** (`cad_new_project`, 0 bodies). Do not
continue a recovered or older Print Kit Tutor. Hide construction planes
before save.

A kit that exports 3MF and cannot make directionless torque is a **fail**.
A kit whose frame out-girths the rotor, whose blades are a straight
fence, or whose only bearing is a short two-land sleeve, is also a
**fail**.

## Grouped expected design

| Group | Number | Why |
|-------|--------|-----|
| Section | NACA 0021, TE ≥ 0.8 mm, root chord > tip | 2026 thick-symmetric band; drafted standing print |
| Rotor | One body: hub + 3 helical blades. X2D-max R=85, span=220, scale 0.4 in the exam | Always some blade working; no tenons |
| Stand | Y-frame + short square stator post | Envelope/rotor ≤ 1.55; no cookie; no tall mast |
| Bearing | Distinct outer-race bushing + 6× printed PIP rollers, min Ø8, large PCD | Tip moment. Hub mounts on the bushing OD. Not a 608. Not a two-land sleeve alone |
| Fits | running +0.40 / slip +0.28 / friction +0.16 | Role-based PLA + 0.4 mm nozzle. Slicer XY hole comp = 0 |
| Envelope | scale 1.0 fits Bambu Lab X2D 256×256×260 with 8 mm margin | `spec.scale` is the source parameter |

Straight H-rotor is the wrong default at this Re: torque ripple and
dead azimuths. Helix is required. Mid-chord stays on the cylinder;
chord stays tangent. Open drafted tips. Do not twist the section in place.

The center is short on purpose. A stout roller pack inside a distinct
bushing takes angular load from the blade tips. The hub friction-mounts
on the bushing OD and sits on the shoulder — it is not the outer race.
Rotational parts print lying down (flanged puck / ring). Only the rotor
stands, and its tips are open.

## Why this page exists

The exam used to accept geometry that was not a turbine: leftover helical
C-buckets, concentric hoop sectors, then a flat plate. Later it accepted
a kit that was a turbine on paper but not a printable assembly: uniform
+0.40 on every hole, three tenoned wings, a tall shaft, a two-land sleeve
that cannot hold tip moment, and no drawing. This page is the research
bar the prompt now enforces.

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

## 2026 section research (cite what you build)

| Source | Year | What it says for this kit |
|--------|------|---------------------------|
| Tirandaz / Rezaeiha line, *Energies* **19**(7) 1615 | 2026 | Coupled t/c, xt/c, LE radius. Favorable low-TSR band **t/c 21–24%**, **xt/c 27.5–35%**, LE index **I ≈ 4.5**. Best in that study: **NACA 0024–4.5/3.5** |
| Range-wide Darrieus CFD + MOP (Joukowsky parameterization) | 2026 | Optimized sections beat NACA 0021 across TSR (~11% mean Cp) |
| Thin vs thick + plain/Gurney/hybrid flaps, *Results in Engineering* | 2026 | Low-Re Darrieus: thick 0021 + hybrid flap can raise Cp; a flap is not a substitute for a section |
| Gu, MacDonald, Tang, *Flow* 6 | 2026 | Adaptive Darrieus–Savonius flap: optional starter, not the wing |

**Printable required section for the exam:** NACA **0021** (t/c = 0.21)
with trailing edge blunt to **≥ 2 nozzles**. Construct 0024–4.5/3.5 if
you can. A rectangle with the same bounding box is a vane.

## Service finish

As-printed FDM skins are rough (Ra often 10–25 µm). That is Cd.

- PLA: sand 400→1000 on blade skins; optional filler primer (consumable,
  not hardware)
- Layer lines **parallel to span** on blades (print the rotor standing)
- Race bores as XY circles (axle / bushing / cage / retainer printed flat)
- Do not vapor-smooth a running fit and then keep the modeled clearance

## Printed roller pack, not a 608

At this size the kit is fully 3D printed. Metal 608 / catalog rollers are
a later table ([ASSEMBLY.md](ASSEMBLY.md)). A short two-land sleeve
cannot take the moment at the blade tips.

| Station | What | Why |
|---------|------|-----|
| Thrust | Axle flange land, 0.20 float under the bushing | Flat land, not a lifted cone |
| Radial + moment | PIP rollers in a cage, large PCD, **bushing ID** = outer race | Short center, stout pack |
| Hub mount | Friction +0.16 on bushing OD; hub sits on the shoulder | Wheel-on-bearing. Not a running fit |
| Stator | Square post on the base (friction in the axle puck) | Print-flat; bushing freewheels |
| Retain | Slip washer on the post covering the open raceway | Not a press |

PLA-on-PLA is a demo spin. Say so. No glue-as-fit.

## Report

The exam writes `Print-Kit-Tutor-design.md` next to the project and
one laid-out `01-kit.3mf` (cartridge is PIP; not the assembled nest).
Materials are **PLA Orange** and **PLA Glow** only. It deletes retired
plates from earlier kits before writing. Required sections: iteration
log, design process, final product, plastic/material cost. Sample:
[PRINT_KIT_REPORT.md](PRINT_KIT_REPORT.md).
