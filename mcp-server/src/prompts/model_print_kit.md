CAD synthesis tutor — design a fully printed, omnidirectional VAWT *assembly* that would actually run, then grade FDM tolerancing.

Spec: scripts/fixtures/print-kit-tutor.spec.json (id fdm-print-vawt).
Contract: docs/agentic/PRINT_KIT_DESIGN.md.
Rerun: npm run test:mcp-print-kit.
Printer: Bambu Lab X2D (256×256×260, 8 mm margin). `spec.scale` shrinks the X2D-max source (exam default 0.4). Clamp rollers ≥ Ø8 / h28, TE ≥ 0.8, walls ≥ 4 nozzles. The cup is a **drum** (land ≥28 mm at exam scale, 70 mm at scale 1.0) on a ≥5 mm plate — not a washer.

Nozzle = {nozzle} mm. Fits are **per role**, not one clearance for every hole:

- running (rollers / races printed as **other** bodies, then assembled): +{nozzle} mm diametral
- PIP (same-plate cage pockets): +0.80 mm (2 nozzles). Assembled running is 0.20/side and **welds** if the roller and cage share a layer
- slip (retainer on the post): +0.28 mm
- friction locate (axle on the square post only): +0.16 mm on a land **above** a 0.80 mm bed lead-in

No FDM press fits. Slicer XY hole compensation stays 0. Thrust is a flat land with 0.20 mm float at **every** running land (flange↔plate, rollers↔cup floor, cup↔retainer). The rotor **is** the housing: a **thick plate** (thrust floor) plus a **tall cup** (outer race) you can see from a 3/4 view, blades growing from that plate. Matching more 8 mm flats is still a pancake stack. Cage height **matches** roller height. The retainer is a washer that covers the open cup (OD between cup ID and cup OD), not a cap through the plate. Every bed-printed functional hole gets a 2-nozzle elephant-foot lead-in. Do **not** print the cup around the PIP rollers on the plate.

This prompt is adversarial. A kit that spins in the grader and fails as a turbine is a FAIL. Read the reject list before you sketch.

## Reject list (automatic fail)

Do not ship any of these. They have already been built. They are not turbines.

- Print-bed scatter / coupon / “kit of parts” that does not assemble
- Separate tenoned wings (the 3 blades and hub are **one rotor body**)
- A tall skinny shaft or sleeve that needs a tower of support
- A short **two-land** sleeve as the only bearing — that cannot take blade-tip moment. Print a large-PCD roller pack
- A **loose bushing sandwich**: separate outer-race ring, postage-stamp flange, unmatched roller / cage / race heights, and no meaningful attach. For a heavy overhung load, thrust + roller live in **one frame** at the **correct matching dimensions**. Do not invent metal 608s to paper over that
- A **washer cup** / postage-stamp land sold as a housing. Matching 8 mm flats to 8 mm rollers still reads as pancakes stacked on the plate. The cup is a **drum** (≥28 mm at exam scale) on a ≥5 mm plate. Look at the solid — do not grade numbers and ship another stack of flats
- A tall cup with the blades still only stuck to the plate. That is a can on a cracker. The overhung moment never reaches the race. Each blade root must **climb the cup wall** (a web as tall as the cup, outside the raceway, into the airfoil). Height-matching cylinders is not a load path
- Assembled spinner whose rotor collides with the stand
- Leftover helical C-loft buckets
- Concentric ring sector / hoop / “scoop” whose concave face points at the axis (no net torque)
- Flat rectangular plate sold as a wing (a plate is a vane, not an airfoil)
- Turntable, lazy Susan, paint wells, or any platter that throws the wing away
- One-sided vane, even blade count that cog-locks, or a rotor that only works in one wind azimuth
- Metal 608 / catalog roller / ball bearings as hidden parts. Fully 3D printed. The outer race is the **rotor cup**, not a distinct bushing the blades cannot attach to
- PIP at assembled running clearance (cage pockets or rollers inside the cup at +0.40). Same-plate gap must be **2 nozzles**
- A friction or running bore on the print bed with no lead-in (elephant foot closes +0.16 and pinches a race)
- Nesting the cup around the PIP rollers on the plate (0.10 mm/side — they fuse)
- Overhung blade roots / a loft that starts mid-hub or from the **top of an arm** (the standing print has no first-layer airfoil). The airfoil draft must continue down and **end on the sit-plane horizontal**
- A tiny hub ring with skinny arms and blades hung off the ends. The rotating mount is a **root plate** out to the blades with an **integral cup** (thrust floor + outer race)
- A closed race you cannot load. The cup is **open-top**: drop the rotor on the axle, then drop the cartridge in from above. Retainer last
- Press fits, same-angle lifted cones (parallel surfaces never touch)
- A report that only says READY TO PRINT
- A fat frame: base envelope > 1.55 × rotor tip diameter (the Ø90 cookie)
- A full cookie plate when a Y-frame (hub + ribs + pads) will hold the stand
- Straight prismatic airfoils (a fence). Helix is required
- The assembled nest exported as the print job (parts still stacked). Layout the kit on one plate first (`solid_move_copy`), then export
- Leftover plates from a previous kit (`01-base` … `05-retainer`, `02-shaft`, `03-hub`, `04-wings`, `05-plate`, `06-bushing`, `07-cap`, or `Print-Kit-Tutor.3mf`). Wipe `Print-Kit-Tutor/` before you write `01-kit.3mf`
- More than two print materials. This kit is **PLA Orange** and **PLA Glow** only

If the solid looks like a broken rim, a fence slat, or a lid, start over.

## Product

A **directionless vertical-axis wind turbine** (VAWT) sized to fill a Bambu Lab X2D at `scale` 1.0. Wind azimuth does not matter. There is no yaw.

Architecture (required): **helical / Gorlov H-Darrieus** with a **symmetric airfoil** section. Symmetric because the blade sees reversing α every revolution. Helical because a straight extrusion is idle for most of the rev. Directionless means: odd blade count, 120° spacing, chord tangent to the cylinder at every station, identical blades, no preferred wind azimuth.

Helix rule: loft a **closed NACA** at ≥2 stations. Mid-chord stays on radius R. Chord stays tangent. Twist ≥45° over the span (spec: 60°). Root at 30°. Open drafted tips — no end cap. First station is the **root plate** (the sit / print plane). Chord drafts toward the tip. The blade bottom is that flat horizontal — not a cut from a surface above.

The center is **short**. Do not build a tall mast. A stout printed roller pack inside the **rotor cup** takes the angular load from the blade tips. The stator is a square post on the Y-frame; the cup freewheels on the rollers. There is no separate bushing for the blades to friction-mount onto.

## 2026 airfoil (required)

Cite a real section. Build that section. Do not invent a “sort of wing.”

2026 VAWT dynamic-stall work (Tirandaz / Rezaeiha line; *Energies* 19(7) 1615, 2026) says **coupled** thickness, thickness-position, and leading-edge radius matter. The favorable low-TSR band is:

- t/c = **21–24%** (thick enough to survive reversing α and to print)
- xt/c = **27.5–35%** (aft of a skinny NACA 0012)
- reduced LE radius index I ≈ **4.5** (not the default I = 6.0)
- Best reported stand-in in that study: **NACA 0024–4.5/3.5** vs a NACA 0018–6.0/3.0 baseline

For this print, the required section is **NACA 0021** (t/c = 0.21) with a **blunt trailing edge ≥ 2 nozzles** ({nozzle} × 2 mm). Draft the chord (root > tip) so the standing print needs little support.

## Grouped expected design (build this, not a remix)

Five **individual** functional parts, then an **assembly** (components + occurrences + joints). Default fully 3D printed.

| Part | What | Print |
|------|------|-------|
| Base | **Y-frame** + short square stator post. One piece. No cookie. | Flat |
| Axle | Flanged inner-race **puck**, square bore (friction on the post). Never a tall skinny shaft | On the flange |
| Rotor | **Root plate** out to the blades, **integral cup** (thrust floor + outer race), 3 helical drafted **NACA 0021** ending on that plate, **one body**, open tips. Plate bore = inner race + running | Standing on the plate |
| Roller cartridge | Cage + ≥6 PIP rollers, min Ø8 / **h28**, large PCD **inside the cup ID**. Cage height = roller height. The cup they sit in is a drum, not a washer | Flat, PIP on the kit plate |
| Retainer | Washer covering the open cup, slip on the post | Flat |

Girth gates: envelope/rotor D ≤ 1.55; span/chord ≥ 2.5; solidity 0.24–0.45. Scale 1.0 must fit the X2D usable bed (240×240×244).

Fits stay role-based. Do not put +{nozzle} on a friction locate and then wonder why nothing is tight.

## Service finish (dynamics)

FDM layer lines are roughness. Unfinished printed airfoils pay Cd.

Specify, in the report, all of:

1. **Print orientation per body.** Rotor standing on the deck so layer lines run **spanwise** and each blade root is on the bed. Axle / cage / retainer / base flat (bores are XY circles)
2. **As-printed Ra class**: assume 10–25 µm as-printed FDM. Target service Ra ≤ 5 µm on blade skins
3. **Finish process (no extra machine parts)**: PLA — sand 400 → 800 → 1000 on blade skins. ABS — acetone vapor 40–50 min class; do not immerse. Do not vapor-smooth a running fit
4. **Forbidden on aero / race faces**: fuzzy skin, fuzzy supports, elephant-foot left on the land
5. **Trailing edge**: blunt ≥ 2 nozzles

## Rotation — printed roller pack in one cup, not a 608

The bearing is a **printed roller cartridge** (PIP rollers in a cage) inside a **rotor cup** at a large pitch circle so it can take the moment at the blade tips. The cup is a **drum you can see** — not another ring of flats. Thrust on the axle flange land (0.20 float under the plate) **and** on the cup floor under the rollers. No metal 608, no 623, no tall two-land sleeve as the only bearing. No loose bushing the blades cannot attach to.

- Inner race: cylindrical OD on the axle puck (printed separately)
- Outer race: **cup ID** (integral to the rotor), height = roller height + 0.20 float, **≥28 mm at exam scale**
- Cup floor / root plate: **≥5 mm** thrust land the rollers sit on; blades grow from this plate
- Cage + rollers are the **PIP cluster** (pockets +0.80). Cage height **equals** roller height. The cage is a spacer; axial capture is the axle flange + retainer
- Do not PIP the rollers inside the cup. Do not close a top inward lip over the rollers
- Square post is the **stator** (friction in the axle, slip in the retainer). The rotor does **not** key to the post

PLA-on-PLA is a demo spin, not a 1000 h bearing. Say that in the report.

## Print and assemble — no additional hardware

No screws, nuts, heat-set inserts, metal shafts, metal bearings, glue as a fit, or rubber bands.

Assembly order: **base → axle → rotor → roller cartridge → retainer**.

Then form a **linked** CAD assembly: `cad_set_focus assembly` (or `cad_set_workspace assembly`). `assembly_create_component` per body that moves — that call already inserts the one root occurrence. **Do not** `assembly_create_occurrence` again. Ground the base. Joints:

- `rigid` axle_sit — axle sits on the base (stator). Square post is friction locate (+0.16), not a press
- `revolute` rotor_spin — cup ID freewheels on the axle inner race. Rollers on both races are **running +0.40**
- `revolute` cage_spin — cage can turn on the same axis
- `revolute` per roller — each PIP roller spins in its pocket
- `rigid` retainer_sit — washer with a **square slip** hole on the post

Pick on-axis circular edges / cylinders (cup ID, axle race, roller axes). Do not pick a blade-spar face — planar centroids yank parts off-axis. `assembly_solution` must be solved without occurrence yanks. Blades stay **one printed body** with the plate (centrifugal + cyclic root bending). The plate **is** the rotating mount — that is how the rotor rotates. Ship an assembly drawing: `cad_drawing_create_sheet` + `cad_drawing_auto_layout` + notes for fits, loads, print orientation, and BOM.

Print each functional part in its own orientation on **one** plate. The roller cartridge is print-in-place (cage + rollers stay together). Save the assembled `.nbcad`, then `solid_move_copy` parts onto the bed and `solid_export_3mf` once as `01-kit`. Appearances: PLA Orange (base, axle, cage, rollers, retainer) and PLA Glow (rotor / cup). Do not export the assembled nest as the print job.

Minimum wall 1.6 mm (4 nozzles). Functional holes are complete XY circles. Disable grid snap. Prefer locked circles. Draw airfoil polylines with **ctrl held** or ortho-snap will square the section into a plate.

## Build (keep the stand, cut unused plastic)

1. prompts/get model_print_kit. cad_list_all_tools. **cad_new_project on a blank document.** `solid_scene` must show **0 bodies** before the first extrude. Do not continue a recovered or older Print Kit Tutor (tan nest, red disc, leftover helix planes). Desktop: File → New, then build — or File → Open the current `Print-Kit-Tutor.nbcad` after this exam writes it. cad_set_document_name Print Kit Tutor
2. Base **Y-frame** + square stator post. One piece, print flat
3. Axle: flange + inner-race cylinder, square bore, print on the flange
4. Rotor: **root plate** (print sit + thrust, ≥5 mm) with an **integral drum** (land ≥28 mm at exam scale). Each blade gets a **web as tall as the cup** from the cup wall (outside the raceway) out to the airfoil — that is the overhung-load path. Loft three helical **NACA 0021** from the plate top (sit plane), open drafted tips. Do not start the loft from a surface above the plate. A tall can with blades only on the plate is still a cracker. The cup **is** the outer race
5. Roller cage + PIP rollers on a large PCD **inside the cup**. Cage height = roller height. Retainer washer over the raceway
6. cad_set_workspace assembly. One `assembly_create_component` per moving body (base, axle, rotor, cage, each roller, retainer — no extra occurrence). Ground the base. Rigid axle_sit + retainer_sit; revolute rotor_spin / cage / rollers on axes (not a spar). cad_set_focus drawing. Sheet + auto-layout + notes
7. cad_set_focus print. set_body_appearance to **PLA Orange** and **PLA Glow** only. solid_export_preflight. Save the assembled `.nbcad`. **Delete** any prior `Print-Kit-Tutor/` 3MFs (and `Print-Kit-Tutor.3mf`). `solid_move_copy` the parts onto one bed (rotor standing on the root plate, others flat). Then `solid_export_3mf` **once** as `01-kit` with every kit body_id. The folder must contain exactly that file.
8. `cad_set_project_visibility`: hide every construction plane (`hidden_datum_plane_ids`) and finished loft sketches (`hidden_sketch_names`). The shipped `.nbcad` must read as the five-part kit, not orange datum stacks.
9. Write the design report. Include role-based fits, scale vs X2D, roller PCD, why the cup is a drum (not a washer), and why a loose bushing sandwich fails an overhung load

## Design report (required deliverable)

Write `%USERPROFILE%/Documents/noBS-CAD/Print-Kit-Tutor-design.md` (and the JSON report beside the project). List the one laid-out `01-kit.3mf`. The exam fails if this is missing, empty, or cost-free, or if retired plates are still on disk.

The report must include:

### 1. Iteration log (what failed, why)

At least the real product faults: scatter, colliding spinner, helical C, hoop sector, turntable, flat plate, straight NACA in a fat cage, uniform +0.40 on every hole, PIP at assembled running clearance, bed-printed friction bore with no lead-in, tenoned separate wings, tall skinny shaft, two-land sleeve that cannot take tip moment, loose bushing sandwich with unmatched heights and no attach.

### 2. Design process

- Architecture (H-Darrieus, short center, why not a tall mast)
- Airfoil citation (2026 sources + the section you actually built)
- Solidity, estimated TSR, estimated Re
- Fit table (running / slip / friction)
- Roller pack in one cup vs two-land sleeve vs 608 vs loose bushing
- Scale parameter vs X2D envelope
- Service-finish plan and print orientation
- Assembly drawing

### 3. Final product

BOM of the five parts, stack, assembly order, how the cup is the housing and how the blades grow from that plate.

### 4. Printing cost — plastic and material

CAD solid volume, estimated print mass (PLA Orange + PLA Glow, 1.24 g/cm³ × print-volume factor 0.42), filament cost at the spec $/kg. Do not invent a $0 kit.

## Grade

Timeline ok. Individual parts, then an assembly. One-piece helical rotor. Role-based fits. Printed roller pack in one cup (no 608, no loose bushing). Parts that should print flat, do. Scale fits an X2D at 1.0. Assembly drawing with notes. 3MF is a PK zip per plate. **Design report with iteration, girth check, and cost exists.**
