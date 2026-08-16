# Print Kit Tutor — design report

Spec `fdm-print-vawt` · Printed VAWT assembly · nozzle 0.4 mm · scale 0.4 of Bambu Lab X2D

Worked example from `npm run test:mcp-print-kit`. Live copy:
`%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-design.md`.
Contract: [PRINT_KIT_DESIGN.md](PRINT_KIT_DESIGN.md).

## 1. Iteration log

| Iteration | Why it failed / what changed |
|-----------|------------------------------|
| Print-bed scatter | Parts did not assemble on one axis. Not a machine. |
| Colliding spinner | Rotor swept the posts. Could not rotate. |
| Helical C-buckets | Leftover loft, not a mount, not an airfoil. |
| Hoop sector r20–r28 | Concentric C. Concave faces the axis. No net torque. |
| Turntable / lazy Susan | Competent bearings, no wing. The frame had nothing to do. |
| Flat plate 12×2.4×32 | A vane, not a 2026 symmetric section. Directionless VAWT needs an airfoil. |
| Straight NACA in a Ø90 cage | Section was right; girth was wrong. Prismatic blades idle most of the rev. |
| Uniform +0.40 on every hole | Easy parts stayed easy; tight locates were sloppy. Role-based running/slip/friction. |
| Tenoned separate wings | Three blades plus a hub is one printed rotor, not a puzzle. |
| Tall skinny shaft + two-land sleeve | Cannot take blade-tip moment and needs a support tower. Short post + large-PCD roller pack. |

## 2. Design process

- **Architecture:** Helical H-Darrieus, directionless (no yaw). Short fixed square post. Hub freewheels on a printed roller pack. No tall mast.
- **Airfoil:** NACA 0021 (t/c 0.21). 2026 band is t/c 21–24%. TE blunt to 0.8 mm. Open drafted tips.
- **Rotor:** one piece, N=3, 60° helix, σ in 0.24–0.45. Envelope/rotor ≤ 1.55.
- **Fits:** assembled running +0.40 (rollers on races), PIP +0.80 (cage pockets), slip +0.28 (retainer), friction +0.16 on a land above a 0.80 mm bed lead-in. Do not nest the bushing around the PIP rollers. Slicer XY hole compensation stays 0. No press. No 608.
- **Bushing:** distinct outer-race ring with an external shoulder. PIP roller cartridge (6× min Ø8) on a large PCD inside the ID. Hub friction-mounts on the OD. A two-land sleeve is not enough for tip moment.
- **Scale:** X2D-max source; exam scale 0.4. Feature floors clamped.
- **Service finish:** rotor standing so layer lines run spanwise; sand PLA 400→1000 on skins.
- **Assembly drawing:** A3 sheet, auto-layout, notes for fits / scale / print / BOM.

## 3. Final product

Six functional parts: base → axle → bushing → roller cartridge → rotor → retainer.

Open in CAD: `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad`

Print plate under `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor\`
(the exam wipes this folder first — no leftover five-plate or
shaft/hub/wings/plate/bushing/cap names):

- `01-kit.3mf` — one plate, laid out. PLA Orange + PLA Glow.

## 4. Printing cost (plastic / material)

Assumptions: Bambu PLA Basic Orange + Bambu PLA Glow, 1.24 g/cm³, $20/kg, print-volume factor 0.42.

At exam scale 0.4 the last green run was ~40.9 cm³ CAD solid, **21.3 g**,
**$0.43** PLA. The exam writes the live figures next to the project.
Electricity and machine time are not priced. No additional hardware.
