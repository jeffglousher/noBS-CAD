Print-kit pipeline — build the printed VAWT assembly.

This prompt is a **repeatable build**. Same spec, same tool sequence, same kit.
It is not an AI-capability exam. The gold path is the exam harness
(`print_kit_tutor` / `scripts/mcp-print-kit-tutor.mjs`), which calls these
tools in order.

Spec: scripts/fixtures/print-kit-tutor.spec.json (id fdm-print-vawt).
Rerun: npm run test:mcp-print-kit.
Printer: Bambu Lab X2D (256×256×260, 8 mm margin). `spec.scale` shrinks the
X2D-max source (exam default 0.4). Feature floors: roller Ø ≥ 8, roller
length ≥ 8, TE ≥ 0.8, walls ≥ 4 nozzles, plate ≥ 3.2 mm, base ≥ 3.2 mm.

Nozzle = {nozzle} mm.

## Product

A directionless helical H-Darrieus (VAWT). Three identical
**NACA 0024-4.5/3.5** blades (t/c 24%, xt/c 35%, LE index 4.5), 120°
apart, 60° helix from a 30° root, chord tangent, mid-chord on radius R.
Open tips with a short taper to a flat landing. Organic fatter root on
the plate top is appearance only; through-plate stump is the sit-plane
chord. Larger root chord / higher solidity for low–medium TSR.

Four printed families, then a linked assembly:

| Part | What | Print |
|------|------|-------|
| Stator | Thin Y-frame + race ring + keeper walls + open top-load fence + constant-pass journal + snap groove. One body | Flat |
| Rotor | Thin root plate (underside = upper thrust race) + 3 helical NACA 0024-4.5/3.5. One body. Plate bore larger than journal pass Ø | Standing on the plate |
| Rollers | 8 radial-axis cylinders, min Ø8, pack height = Ø, PCD under the blade roots. Drop into top-load slots | Standing (axis Z) |
| Retainer | Clocked C-clip (D-hole + C-gap) in the journal groove, 0.20 above the plate | Flat |

Plastics: **PLA Basic Orange** (stator, rollers, retainer) and **PLA Glow Green** (rotor).

Girth: envelope/rotor D ≤ 1.55; span/chord ≥ 2.5; solidity 0.24–0.45.
Scale 1.0 fits the X2D usable bed (240×240×244).

## Fits

Role-based. Slicer XY hole compensation stays 0.

- running (rollers on races, separate bodies): +{nozzle} mm diametral
- top-load slots: roller Ø + running + 2 nozzles
- slip (clip D-hole in the groove): +0.28 mm
- 0.20 mm axial float at every running land
- 2-nozzle elephant-foot lead-in on every bed-printed functional hole

Fence height is below pack height. Fence ID is looser than the plate bore
(spacer). Keepers survive the top-load cut. Journal is a constant pass Ø
smaller than the plate bore so the rotor drops on.

## Pipeline

Replay this order. Prefer locked circles. Disable grid snap. Draw airfoil
polylines with **ctrl held**.

1. `cad_new_project` on a **blank document** (`solid_scene` shows 0 bodies).
   `cad_set_document_name` Print Kit Tutor.
2. Stator: Y-frame + race ring + keepers + fence + constant journal + groove.
   Print-flat. `solid_extrude` / cuts for slots, groove, and D-flat.
3. Rotor: root plate, airfoil through the plate, `solid_loft` three helical
   NACA 0024-4.5/3.5 from the organic root to a tapered flat landing.
4. Eight radial-axis rollers. Top-load slots on the stator.
5. C-clip: D-hole + C-gap, print-flat.
6. `cad_set_workspace` assembly. One `assembly_create_component` per moving
   body (that call inserts the root occurrence). Ground the stator.
   - `revolute` rotor_spin — plate bore on the journal, about Z
   - `revolute` per roller — about its radial axis
   - `rigid` retainer_sit — clip in the groove
   Pick circular edges or cylinders. `assembly_solution` stays solved.
7. Drawing: `cad_drawing_create_sheet` + `cad_drawing_auto_layout` + notes
   (fits, scale, print orientation, BOM).
8. Print: `set_body_appearance` (orange / glow). `solid_export_preflight`.
   Save the assembled `.nbcad`. `solid_move_copy` onto one print plate
   (rotor and rollers standing, others flat). `solid_export_3mf` once as
   `01-kit`.
9. `cad_set_project_visibility`: hide construction planes and finished loft
   sketches.
10. Write the design report next to the project: architecture, airfoil,
    fit table, stack, BOM, print orientation, service finish, plastic cost.

Assembly order of the physical kit: stator → rollers into the slots →
drop rotor over the journal → snap C-clip.

## Service finish

Rotor standing so layer lines run spanwise. Stator and clip flat (bores
are XY circles). Rollers standing. Sand PLA skins 400→1000. Trailing edge
blunt ≥ 2 nozzles. PLA-on-PLA is a demo spin.

## Report

`%USERPROFILE%/Documents/noBS-CAD/Print-Kit-Tutor-design.md`

Include: architecture, NACA 0024-4.5/3.5 citation, solidity, fit table, thrust-pack
stack, scale vs X2D, print orientation, BOM, CAD volume, print mass, filament
cost (PLA 1.24 g/cm³ × print-volume factor 0.42 × spec $/kg).
