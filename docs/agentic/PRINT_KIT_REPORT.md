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
| Overhung blade roots | Loft started mid-hub. First layers of a standing print were air. Root plate is the sit plane; roots on the bed. |
| Tiny ring + blades from the surface above | Skinny arms, loft from the arm top. Root plate out to the blades; loft from the sit plane. |
| Loose bushing sandwich | Separate ring, unmatched roller/cage heights, no attach for an overhung load. |
| Washer cup / pancake stack | Matching 8 mm flats still read as cylinders on the plate. Height-matching flats is not a bearing. |
| Tall drum / can on a cracker | A 28 mm orange tower with webs climbing the wall is a journal you can see from the side. Thin flat thrust under the plate. |
| Tall skinny shaft + two-land sleeve | Cannot take blade-tip moment and needs a support tower. Short post + large-PCD thin thrust. |
| Separate axle disk + cage disk | Two flats that should be one stator. Extra plastic and a rubbing washer. Merge Y-frame + race + open fence + journal. Top-load rollers. Clocked C-snap on the journal shoulder. |
| Cookie race under the Y-frame | One Ø74 disk under the rollers reprinted the plastic the merge was supposed to drop. | Race is a ring where rollers contact. Fence sits on that ring. Y-frame stays open. |

## 2. Design process

- **Architecture:** Helical H-Darrieus, directionless (no yaw). One printed stator (Y-frame + race ring + open fence + journal). Thin flat thrust under the plate. No tall mast. No tall drum. No cookie disk. No separate axle puck + cage disk.
- **Airfoil:** NACA 0021 (t/c 0.21). 2026 band is t/c 21–24%. TE blunt to 0.8 mm. Open drafted tips.
- **Rotor:** one piece — root plate (upper thrust race) + print arms + root stumps + 3 helical NACAs from the plate. N=3, 60° helix, σ in 0.24–0.45. Envelope/rotor ≤ 1.55.
- **Fits:** assembled running +0.40 (rollers on races), PIP +0.80 (cage pockets), slip +0.28 (retainer), friction +0.16 on a land above a 0.80 mm bed lead-in. Cage height matches roller height. Do not nest the plate around the PIP rollers. Slicer XY hole compensation stays 0. No press. No 608.
- **Thrust pack:** thin PIP rollers (6× min Ø8 / h3.2) on a large PCD between the axle flange and the plate underside. Short journal centers. A two-land sleeve, a loose bushing, a washer stack, or a tall drum is not the bearing.
- **Scale:** X2D-max source; exam scale 0.4. Feature floors clamped.
- **Service finish:** rotor standing on the deck so layer lines run spanwise; sand PLA 400→1000 on skins.
- **Assembly drawing:** A3 sheet, auto-layout, notes for fits / scale / print / BOM.

## 3. Final product

Three printed families: stator → rollers into top-load slots → rotor → snap retainer.

Open in CAD: `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad`

Print plate under `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor\`
(the exam wipes this folder first — no leftover five-plate or
shaft/hub/wings/plate/bushing/cap names):

- `01-kit.3mf` — one plate, laid out. PLA Orange + PLA Glow.

## 4. Printing cost (plastic / material)

Assumptions: Bambu PLA Basic Orange + Bambu PLA Glow, 1.24 g/cm³, $20/kg, print-volume factor 0.42.

At exam scale 0.4 the last green run was ~73.6 cm³ CAD solid, **38.3 g**,
**$0.77** PLA (race ring, not a cookie disk). The exam writes the live
figures next to the project.
Electricity and machine time are not priced. No additional hardware.
