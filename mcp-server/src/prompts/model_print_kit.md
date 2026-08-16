CAD synthesis tutor — design a fully printed, omnidirectional VAWT *assembly* that would actually run, then grade FDM tolerancing.

Spec: scripts/fixtures/print-kit-tutor.spec.json (id fdm-print-vawt).
Contract: docs/agentic/PRINT_KIT_DESIGN.md.
Rerun: npm run test:mcp-print-kit.
Printer: Bambu Lab X2D (256×256×260, 8 mm margin). `spec.scale` shrinks the X2D-max source (exam default 0.4). Clamp rollers ≥ Ø8, TE ≥ 0.8, walls ≥ 4 nozzles.

Nozzle = {nozzle} mm. Fits are **per role**, not one clearance for every hole:

- running (rollers / races): +{nozzle} mm diametral
- slip (retainer on the post): +0.28 mm
- friction locate (axle on the square post): +0.16 mm

No FDM press fits. Slicer XY hole compensation stays 0. Thrust is a flat land with 0.20 mm float.

This prompt is adversarial. A kit that spins in the grader and fails as a turbine is a FAIL. Read the reject list before you sketch.

## Reject list (automatic fail)

Do not ship any of these. They have already been built. They are not turbines.

- Print-bed scatter / coupon / “kit of parts” that does not assemble
- Separate tenoned wings (the 3 blades and hub are **one rotor body**)
- A tall skinny shaft or sleeve that needs a tower of support
- A short **two-land** sleeve as the only bearing — that cannot take blade-tip moment. Print a large-PCD roller pack
- Assembled spinner whose rotor collides with the stand
- Leftover helical C-loft buckets
- Concentric ring sector / hoop / “scoop” whose concave face points at the axis (no net torque)
- Flat rectangular plate sold as a wing (a plate is a vane, not an airfoil)
- Turntable, lazy Susan, paint wells, or any platter that throws the wing away
- One-sided vane, even blade count that cog-locks, or a rotor that only works in one wind azimuth
- Metal 608 / catalog roller / ball bearings as hidden parts. Fully 3D printed. The bushing **is** the printed roller cartridge
- Press fits, same-angle lifted cones (parallel surfaces never touch)
- A report that only says READY TO PRINT
- A fat frame: base envelope > 1.55 × rotor tip diameter (the Ø90 cookie)
- A full cookie plate when a Y-frame (hub + ribs + pads) will hold the stand
- Straight prismatic airfoils (a fence). Helix is required
- A single assembled 3MF sold as the print job. Separate print plates (the roller cartridge is one PIP plate)
- Leftover plates from a previous kit (`02-shaft`, `03-hub`, `04-wings`, `05-plate`, `06-bushing`, `07-cap`, or `Print-Kit-Tutor.3mf`). Wipe `Print-Kit-Tutor/` before you write the five current plates

If the solid looks like a broken rim, a fence slat, or a lid, start over.

## Product

A **directionless vertical-axis wind turbine** (VAWT) sized to fill a Bambu Lab X2D at `scale` 1.0. Wind azimuth does not matter. There is no yaw.

Architecture (required): **helical / Gorlov H-Darrieus** with a **symmetric airfoil** section. Symmetric because the blade sees reversing α every revolution. Helical because a straight extrusion is idle for most of the rev. Directionless means: odd blade count, 120° spacing, chord tangent to the cylinder at every station, identical blades, no preferred wind azimuth.

Helix rule: loft a **closed NACA** at ≥2 stations. Mid-chord stays on radius R. Chord stays tangent. Twist ≥45° over the span (spec: 60°). Root at 30°. Open drafted tips — no end cap. Print the rotor standing on the hub.

The center is **short**. Do not build a tall mast. A stout printed roller pack at the hub takes the angular load from the blade tips. The stator is a square post on the Y-frame; the hub freewheels on the rollers.

## 2026 airfoil (required)

Cite a real section. Build that section. Do not invent a “sort of wing.”

2026 VAWT dynamic-stall work (Tirandaz / Rezaeiha line; *Energies* 19(7) 1615, 2026) says **coupled** thickness, thickness-position, and leading-edge radius matter. The favorable low-TSR band is:

- t/c = **21–24%** (thick enough to survive reversing α and to print)
- xt/c = **27.5–35%** (aft of a skinny NACA 0012)
- reduced LE radius index I ≈ **4.5** (not the default I = 6.0)
- Best reported stand-in in that study: **NACA 0024–4.5/3.5** vs a NACA 0018–6.0/3.0 baseline

For this print, the required section is **NACA 0021** (t/c = 0.21) with a **blunt trailing edge ≥ 2 nozzles** ({nozzle} × 2 mm). Draft the chord (root > tip) so the standing print needs little support.

## Grouped expected design (build this, not a remix)

Five **individual** functional parts, then an **assembly** (components + occurrences + a revolute). Default fully 3D printed.

| Part | What | Print |
|------|------|-------|
| Base | **Y-frame** + short square stator post. One piece. No cookie. | Flat |
| Axle | Flanged inner-race **puck**, square bore (friction on the post). Never a tall skinny shaft | On the flange |
| Rotor | Hub + 3 helical drafted **NACA 0021** blades, **one body**, open tips | Standing on the hub |
| Roller cartridge | Cage + ≥6 PIP rollers, min Ø8, large PCD under/in the hub | Flat, one print plate |
| Retainer | Flat ring, slip on the post | Flat |

Girth gates: envelope/rotor D ≤ 1.55; span/chord ≥ 2.5; solidity 0.24–0.45. Scale 1.0 must fit the X2D usable bed (240×240×244).

Fits stay role-based. Do not put +{nozzle} on a friction locate and then wonder why nothing is tight.

## Service finish (dynamics)

FDM layer lines are roughness. Unfinished printed airfoils pay Cd.

Specify, in the report, all of:

1. **Print orientation per body.** Rotor standing so layer lines run **spanwise**. Axle / cage / retainer / base flat (bores are XY circles)
2. **As-printed Ra class**: assume 10–25 µm as-printed FDM. Target service Ra ≤ 5 µm on blade skins
3. **Finish process (no extra machine parts)**: PLA — sand 400 → 800 → 1000 on blade skins. ABS — acetone vapor 40–50 min class; do not immerse. Do not vapor-smooth a running fit
4. **Forbidden on aero / race faces**: fuzzy skin, fuzzy supports, elephant-foot left on the land
5. **Trailing edge**: blunt ≥ 2 nozzles

## Rotation — printed roller pack, not a 608

The bushing is a **printed roller cartridge** (PIP rollers in a cage) at a large pitch circle so it can take the moment at the blade tips. Thrust on the axle flange land (0.20 float). No metal 608, no 623, no tall two-land sleeve as the only bearing.

- Inner race: cylindrical OD on the axle puck
- Outer race: hub bore
- Cage sits inside the hub, around the race
- Square post is the **stator** (friction in the axle, slip in the retainer). The hub does **not** key to the post
- Retain with the flange + retainer ring. Not interference

PLA-on-PLA is a demo spin, not a 1000 h bearing. Say that in the report.

## Print and assemble — no additional hardware

No screws, nuts, heat-set inserts, metal shafts, metal bearings, glue as a fit, or rubber bands.

Assembly order: **base → axle → roller cartridge → rotor → retainer**.

Then form the CAD assembly: `cad_set_focus assembly` (or `cad_set_workspace assembly`), `assembly_create_component` per part, `assembly_create_occurrence`, ground the base, `assembly_create_joint` revolute on the axis. Ship an assembly drawing: `cad_drawing_create_sheet` + `cad_drawing_auto_layout` + notes for fits, scale, print orientation, and BOM.

Print each functional part in its own orientation. The roller cartridge is print-in-place (cage + rollers, one plate). Export five print-plate 3MFs (`solid_export_3mf` + `body_ids`). Do not export the assembled nest as the print job.

Minimum wall 1.6 mm (4 nozzles). Functional holes are complete XY circles. Disable grid snap. Prefer locked circles. Draw airfoil polylines with **ctrl held** or ortho-snap will square the section into a plate.

## Build (keep the stand, cut unused plastic)

1. prompts/get model_print_kit. cad_list_all_tools. cad_new_project. cad_set_document_name Print Kit Tutor
2. Base **Y-frame** + square stator post. One piece, print flat
3. Axle: flange + inner-race cylinder, square bore, print on the flange
4. Rotor: hub ring (outer race) JOIN three helical **NACA 0021** lofts, open drafted tips, print standing
5. Roller cage + PIP rollers on a large PCD. Retainer ring
6. cad_set_focus assembly. Components / occurrences / revolute. cad_set_focus drawing. Sheet + auto-layout + notes
7. cad_set_focus print. set_body_appearance. solid_export_preflight. **Delete** any prior `Print-Kit-Tutor/` 3MFs (and `Print-Kit-Tutor.3mf`). Then `solid_export_3mf` **once per print plate** with body_ids (base, axle, rotor, cartridge, retainer) only. The folder must contain exactly those five files.
8. Write the design report. Include role-based fits, scale vs X2D, roller PCD, and why the rotor is one piece

## Design report (required deliverable)

Write `%USERPROFILE%/Documents/noBS-CAD/Print-Kit-Tutor-design.md` (and the JSON report beside the project). List the five print-plate 3MFs. The exam fails if this is missing, empty, or cost-free, or if retired plates are still on disk.

The report must include:

### 1. Iteration log (what failed, why)

At least the real product faults: scatter, colliding spinner, helical C, hoop sector, turntable, flat plate, straight NACA in a fat cage, uniform +0.40 on every hole, tenoned separate wings, tall skinny shaft, two-land sleeve that cannot take tip moment.

### 2. Design process

- Architecture (H-Darrieus, short center, why not a tall mast)
- Airfoil citation (2026 sources + the section you actually built)
- Solidity, estimated TSR, estimated Re
- Fit table (running / slip / friction)
- Roller pack vs two-land sleeve vs 608
- Scale parameter vs X2D envelope
- Service-finish plan and print orientation
- Assembly drawing

### 3. Final product

BOM of the five parts, stack, assembly order, how the hub freewheels.

### 4. Printing cost — plastic and material

CAD solid volume, estimated print mass (Bambu PLA Basic 1.24 g/cm³ × print-volume factor 0.42), filament cost at the spec $/kg. Do not invent a $0 kit.

## Grade

Timeline ok. Individual parts, then an assembly. One-piece helical rotor. Role-based fits. Printed roller pack (no 608). Parts that should print flat, do. Scale fits an X2D at 1.0. Assembly drawing with notes. 3MF is a PK zip per plate. **Design report with iteration, girth check, and cost exists.**
