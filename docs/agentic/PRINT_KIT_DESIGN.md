# Print-kit tutor — design contract (2026)

Build contract for pipeline #1. The recipe is `prompts/get model_print_kit`
(`mcp-server/src/prompts/model_print_kit.md`). Numbers live in
`scripts/fixtures/print-kit-tutor.spec.json`. Fits and closed faults:
[PRINT_KIT_GDT.md](PRINT_KIT_GDT.md). Why this section and why it
takes wind from any heading (math, figures, citations):
[PRINT_KIT_AERO.md](PRINT_KIT_AERO.md).
Which 3MF is live:
[PRINT_KIT_FILES.md](PRINT_KIT_FILES.md).

Start from a **blank document** (`cad_new_project`, 0 bodies). Do not
continue a recovered or older Print Kit Tutor. Hide construction planes
before save.

## Grouped expected design

| Group | Number | Why |
|-------|--------|-----|
| Section | NACA 0024-4.5/3.5, TE ≥ 0.8 mm, root chord > tip | Low-TSR Tirandaz/Rezaeiha section; drafted standing print |
| Rotor | One body: root plate out to the blades (underside = upper thrust race), 3 helical blades ending on that plate. X2D-max R=85, span=236, scale 0.4 in the exam | Always some blade working; no tenons; no mid-air roots; no skinny-arm hangers |
| Stand | One stator: Y-frame + race **ring** + open top-load fence + short journal | Envelope/rotor ≤ 1.55; no cookie disk; no tall mast; no separate axle disk |
| Bearing | **Thin flat thrust** under the **blade roots**: 8× printed **radial-axis** rollers, min Ø8, pack height = Ø, large PCD. Top-load slots. **Keeper walls**. Fence ID looser than the plate bore. Thin plate (≥3.2 mm) with **organic airfoil roots**. Constant journal the plate can drop over. Clocked C-clip in a groove | Rotation about Z plus overturning as a couple under the roots |
| Fits | running +0.40 / PIP +0.80 / slip +0.28 / friction +0.16 | Role-based **and** same-plate vs assembled. 0.80 mm bed lead-in. Slicer XY hole comp = 0 |
| Envelope | scale 1.0 fits Bambu Lab X2D 256×256×260 with 8 mm margin | `spec.scale` is the source parameter |

Straight H-rotor is the wrong default at this Re: torque ripple and
dead azimuths. Helix is required. Mid-chord stays on the cylinder;
chord stays tangent. Open drafted tips. Do not twist the section in place.

The center is short on purpose. A **thin flat thrust** under the
**blade roots** takes rotation about Z. Overturning from the blade tips
is a couple across that pitch circle. The fence is a spacer (ID looser
than the plate bore) and sits on the race ring only. The lower race is
a **ring** under the rollers, joined to the Y-frame. The stator and
retainer print flat. Rollers print **standing** and drop into top-load
slots (axis radial). The rotor stands on that plate. Blade roots are XY
airfoils through the plate, with an organic blend on the top face and a
short taper to a flat tip landing. Keepers capture the rollers radially.
The retainer is a clocked C-clip in a journal groove — the plate drops
over a constant journal, then the clip snaps on. Clip families:
[PRINT_KIT_GDT.md](PRINT_KIT_GDT.md).

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

**Printable required section:** NACA **0024-4.5/3.5** (t/c = 0.24,
xt/c = 0.35, LE index I = 4.5) with trailing edge blunt to **≥ 2 nozzles**.
Organic root blend and tip landing are appearance only
([PRINT_KIT_ORGANIC_REFERENCE.md](PRINT_KIT_ORGANIC_REFERENCE.md)).
A rectangle with the same bounding box is a vane.

## Service finish

As-printed FDM skins are rough (Ra often 10–25 µm). That is Cd.

- PLA: sand 400→1000 on blade skins; optional filler primer (consumable,
  not hardware)
- Layer lines **parallel to span** on blades (print the rotor standing)
- Race bores as XY circles (axle / cage / retainer printed flat; plate bore is on the standing rotor)
- Do not vapor-smooth a running fit and then keep the modeled clearance

## Printed roller pack, not a 608

At this size the kit is fully 3D printed. Metal 608 / catalog rollers are
a later table ([ASSEMBLY.md](ASSEMBLY.md)). A short two-land sleeve
cannot take the moment at the blade tips.

| Station | What | Why |
|---------|------|-----|
| Thrust | Stator race (lower) ↔ thin rollers ↔ plate underside (upper), 0.20 float each land | Flat land, not a lifted cone, not a tall drum |
| Overturning | Same pack, large PCD **under the blade roots** | Couple across the disk. Width, not height. An inboard PCD leaves the plate as a cracker |
| Stator | One body: Y-frame + race ring + open fence + journal | Print-flat; plate freewheels on the short journal |
| Retain | Clocked C-clip in a journal groove | Plate drops on; clip snaps on and pulls off; does not rub the rotor |

PLA-on-PLA is a demo spin. Say so. No glue-as-fit.

## Report

The exam writes `Print-Kit-Tutor-design.md` next to the project and
one laid-out `01-kit.3mf` (rollers print standing; not the assembled nest).
Materials are **PLA Orange** and **PLA Glow** only. It deletes retired
plates from earlier kits before writing. Required sections: iteration
log, design process, final product, plastic/material cost. Sample:
[PRINT_KIT_REPORT.md](PRINT_KIT_REPORT.md).
