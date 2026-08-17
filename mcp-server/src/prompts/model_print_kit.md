CAD synthesis tutor — design a fully printed, omnidirectional VAWT *assembly* that would actually run, then grade FDM tolerancing.

Spec: scripts/fixtures/print-kit-tutor.spec.json (id fdm-print-vawt).
Contract: docs/agentic/PRINT_KIT_DESIGN.md.
Rerun: npm run test:mcp-print-kit.
Printer: Bambu Lab X2D (256×256×260, 8 mm margin). `spec.scale` shrinks the X2D-max source (exam default 0.4). Clamp rollers ≥ Ø8, roller length ≥ 8, TE ≥ 0.8, walls ≥ 4 nozzles. The bearing is a **thin flat thrust** under the **blade roots** (large PCD, **radial-axis** rollers, pack height = roller Ø) — not a washer stack, not standing-Z pucks, not an inboard pack that leaves the plate as a cantilever, and not a tall drum.

Nozzle = {nozzle} mm. Fits are **per role**, not one clearance for every hole:

- running (rollers / races printed as **other** bodies, then assembled): +{nozzle} mm diametral
- top-load slots: roller Ø + running + 2 nozzles so rollers drop in from above without support
- PIP (same-plate cluster): +0.80 mm (2 nozzles). This kit does **not** PIP the rollers — a lying OD prints as layers
- slip (clocked C-snap D-hole on the journal neck): +0.28 mm

No FDM press fits. Slicer XY hole compensation stays 0. Thrust is a flat land with 0.20 mm float at **every** running land (stator race↔rollers, rollers↔plate). The machine's job is **rotation about Z**. A **thin, wide** printed thrust pack under the plate provides that: cylinders whose **axis is radial** so they roll in the circumferential direction. Overturning from tall blades is a couple across a large pitch circle — not a tall journal you can see from the side. Fence height is **below** pack height so rollers touch both races. The retainer is a **clocked C-snap** on the journal shoulder — it does **not** rub the rotor. Every bed-printed functional hole gets a 2-nozzle elephant-foot lead-in. Do **not** print the plate around the rollers on the bed. Do **not** keep a separate axle disk and cage disk.

This prompt is adversarial. A kit that spins in the grader and fails as a turbine is a FAIL. Read the reject list before you sketch.

## Reject list (automatic fail)

Do not ship any of these. They have already been built. They are not turbines.

- Print-bed scatter / coupon / “kit of parts” that does not assemble
- Separate tenoned wings (the 3 blades and hub are **one rotor body**)
- A tall skinny shaft or sleeve that needs a tower of support
- A short **two-land** sleeve as the only bearing — that cannot take blade-tip moment. Print a large-PCD roller pack
- A **loose bushing sandwich**: separate outer-race ring, postage-stamp flange, unmatched roller / cage / race heights, and no meaningful attach. Do not invent metal 608s to paper over that
- A **washer cup** / pancake stack. Matching 8 mm flats to 8 mm rollers still reads as pancakes stacked on the plate. Height-matching cylinders is not a bearing
- **Standing-Z pucks** (axis = Z, end faces sliding on the races). That is not rolling under −Z. Pack height is the roller diameter; axes are **radial**
- **Inboard pack / cage as journal.** A PCD at ~58% of the plate leaves the blade roots cantilevered on 5 mm PLA — the couple never reaches the race. A cage ID tighter than the plate bore steals the radial land. The pack outer land must reach the blade radius. Fence ID is looser than the plate bore (spacer)
- **Separate axle disk + cage disk.** Two flats that should be one stator. Extra plastic, extra assembly, and a rubbing washer. Merge Y-frame + race + open fence + journal
- **Pickup cartridge.** Open-top windows are how the rollers load from above and still touch both races. Drop rollers into the stator slots, then the rotor. Do not claim you can pick up a preloaded cage
- **Tangent-axis** rollers. Relative motion at the race is circumferential; a tangent axis rolls inward/outward
- A **tall cup / tall drum** in the hub (an orange tower, “land ≥28 mm”, moment webs climbing a wall). That is a journal you can see from the side. We do not need a tall system. A thin flat thrust under the plate handles rotation for the tall airfoils
- Assembled spinner whose rotor collides with the stand
- Leftover helical C-loft buckets
- Concentric ring sector / hoop / “scoop” whose concave face points at the axis (no net torque)
- Flat rectangular plate sold as a wing (a plate is a vane, not an airfoil)
- Turntable, lazy Susan, paint wells, or any platter that throws the wing away
- One-sided vane, even blade count that cog-locks, or a rotor that only works in one wind azimuth
- Metal 608 / catalog roller / ball bearings as hidden parts. Fully 3D printed
- PIP a lying roller (the OD prints as layers) or nest rollers under the plate at +0.40 (0.10 mm/side — they fuse)
- A friction or running bore on the print bed with no lead-in (elephant foot closes +0.16 and pinches a race)
- Nesting the plate around the rollers on the bed
- Overhung blade roots / a loft that starts mid-hub or from the **top of an arm** (the standing print has no first-layer airfoil). The airfoil draft must continue down and **end on the sit-plane horizontal**. No rectangular print arms or spars
- A tiny hub ring with skinny arms and blades hung off the ends. The rotating mount is a **root plate** out to the blades. The plate underside is the upper thrust race
- A closed race you cannot load. Cage on the flange, rollers into the windows, then the rotor. Retainer last
- Press fits, same-angle lifted cones (parallel surfaces never touch)
- A report that only says READY TO PRINT
- A fat frame: base envelope > 1.55 × rotor tip diameter (the Ø90 cookie)
- A full cookie plate when a Y-frame (hub + ribs + pads) will hold the stand
- Straight prismatic airfoils (a fence). Helix is required
- The assembled nest exported as the print job (parts still stacked). Layout the kit on one plate first (`solid_move_copy`), then export
- Leftover plates from a previous kit (`01-base` … `05-retainer`, `02-shaft`, `03-hub`, `04-wings`, `05-plate`, `06-bushing`, `07-cap`, or `Print-Kit-Tutor.3mf`). Wipe `Print-Kit-Tutor/` before you write `01-kit.3mf`
- More than two print materials. This kit is **PLA Orange** and **PLA Glow** only

If the solid looks like a broken rim, a fence slat, a lid, or an orange tower in the hub, start over.

## Product

A **directionless vertical-axis wind turbine** (VAWT) sized to fill a Bambu Lab X2D at `scale` 1.0. Wind azimuth does not matter. There is no yaw.

Architecture (required): **helical / Gorlov H-Darrieus** with a **symmetric airfoil** section. Symmetric because the blade sees reversing α every revolution. Helical because a straight extrusion is idle for most of the rev. Directionless means: odd blade count, 120° spacing, chord tangent to the cylinder at every station, identical blades, no preferred wind azimuth.

Helix rule: loft a **closed NACA** at ≥2 stations. Mid-chord stays on radius R. Chord stays tangent. Twist ≥45° over the span (spec: 60°). Root at 30°. Open drafted tips — no end cap. First station is the **root plate** (the sit / print plane). Chord drafts toward the tip. The blade bottom is that flat horizontal — not a cut from a surface above.

The center is **short**. Do not build a tall mast. Do not build a tall drum. A **thin flat thrust** under the **blade roots** takes rotation about Z and the overturning couple from the blade tips. The stator is **one printed body**: Y-frame + lower race + open top-load fence + short journal. The plate freewheels on the pack. The plate bore is the radial land; the fence is a spacer. There is no separate axle puck, cage disk, or bushing.

## 2026 airfoil (required)

Cite a real section. Build that section. Do not invent a “sort of wing.”

2026 VAWT dynamic-stall work (Tirandaz / Rezaeiha line; *Energies* 19(7) 1615, 2026) says **coupled** thickness, thickness-position, and leading-edge radius matter. The favorable low-TSR band is:

- t/c = **21–24%** (thick enough to survive reversing α and to print)
- xt/c = **27.5–35%** (aft of a skinny NACA 0012)
- reduced LE radius index I ≈ **4.5** (not the default I = 6.0)
- Best reported stand-in in that study: **NACA 0024–4.5/3.5** vs a NACA 0018–6.0/3.0 baseline

For this print, the required section is **NACA 0021** (t/c = 0.21) with a **blunt trailing edge ≥ 2 nozzles** ({nozzle} × 2 mm). Draft the chord (root > tip) so the standing print needs little support.

## Grouped expected design (build this, not a remix)

Three printed families, then an **assembly** (components + occurrences + joints). Default fully 3D printed.

| Part | What | Print |
|------|------|-------|
| Stator | **Y-frame** + race ring + open top-load fence + **constant** journal + snap groove. One piece. No cookie. No fat shoulder/bead | Flat |
| Rotor | **Root plate** out to the blades (underside = upper thrust race), 3 helical drafted **NACA 0021** ending on that plate, **one body**, open tips. Plate bore > journal pass Ø so it **drops on** | Standing on the plate |
| Rollers | ≥6 **radial-axis** cylinders, min Ø8, pack height = Ø, large PCD **under the blade roots**. Drop into top-load slots | Standing (axis Z) |
| Retainer | Clocked **C-clip** (D-hole + C-gap). Snaps into the journal groove, 0.20 above the plate — does **not** rub the rotor. Pull to remove | Flat |

Girth gates: envelope/rotor D ≤ 1.55; span/chord ≥ 2.5; solidity 0.24–0.45. Scale 1.0 must fit the X2D usable bed (240×240×244).

Fits stay role-based. Do not put +{nozzle} on a friction locate and then wonder why nothing is tight.

## Service finish (dynamics)

FDM layer lines are roughness. Unfinished printed airfoils pay Cd.

Specify, in the report, all of:

1. **Print orientation per body.** Rotor standing on the deck so layer lines run **spanwise** and each blade root is on the bed. Stator / retainer flat (bores are XY circles). Rollers standing
2. **As-printed Ra class**: assume 10–25 µm as-printed FDM. Target service Ra ≤ 5 µm on blade skins
3. **Finish process (no extra machine parts)**: PLA — sand 400 → 800 → 1000 on blade skins. ABS — acetone vapor 40–50 min class; do not immerse. Do not vapor-smooth a running fit
4. **Forbidden on aero / race faces**: fuzzy skin, fuzzy supports, elephant-foot left on the land
5. **Trailing edge**: blunt ≥ 2 nozzles

## Rotation — thin flat thrust, not a 608, not a tall drum

The bearing is a **printed thrust pack** (radial-axis rollers in a cage) on a **large pitch circle under the blade roots**. That is a normal 2026 printed turntable: width takes the moment; height is the roller diameter. No metal 608, no 623, no tall two-land sleeve, no orange tower, no standing-Z pucks, no inboard cracker.

- Lower race: **stator** (Y-frame + race **ring** under the rollers — not a cookie disk)
- Rolling elements: ≥6 cylinders, **axis radial** (e_r), diameter = pack height, min Ø8. Print standing (circular layers); drop into top-load slots
- Upper race: **plate underside**
- Journal: short inner-race cylinder through the pack / plate — centering only. **Constant pass Ø** (nothing fatter than the plate bore). Undercut snap groove + D-flat **above** the plate only
- Top-load slots are **running + two nozzles**. Fence height is below pack height. The fence is a spacer (ID looser than the plate bore); axial capture is the race + clocked C-clip. Not a pickup cartridge
- Do not PIP a lying roller. Do not nest rollers under the plate. Do not close a top inward lip over the rollers
- Do not print a fat shoulder or snap bead the plate cannot pass — that is an hourglass you cannot assemble
- The rotor does **not** key to the journal. The C-clip clocks to one orientation, snaps on, and pulls off for service
- Plastics: **PLA Basic Orange** and **PLA Glow Green** only

PLA-on-PLA is a demo spin, not a 1000 h bearing. Say that in the report.

## Print and assemble — no additional hardware

No screws, nuts, heat-set inserts, metal shafts, metal bearings, glue as a fit, or rubber bands.

Assembly order: **stator → rollers into the top-load slots → drop rotor over the journal → snap C-clip into the groove**.

Then form a **linked** CAD assembly: `cad_set_focus assembly` (or `cad_set_workspace assembly`). `assembly_create_component` per body that moves — that call already inserts the one root occurrence. **Do not** `assembly_create_occurrence` again. Ground the stator. Joints:

- `revolute` rotor_spin — plate bore freewheels on the short journal. Rollers on both races are **running +0.40**
- `revolute` per roller — each roller spins about its **radial** axis in its top-load slot
- `rigid` retainer_sit — clocked C-snap on the journal shoulder. It does **not** rub the rotor

Pick on-axis circular edges / cylinders (plate bore, axle journal, roller axes). Do not pick a blade-spar face — planar centroids yank parts off-axis. `assembly_solution` must be solved without occurrence yanks. Blades stay **one printed body** with the plate (centrifugal + cyclic root bending). The plate **is** the rotating mount — that is how the rotor rotates. Ship an assembly drawing: `cad_drawing_create_sheet` + `cad_drawing_auto_layout` + notes for fits, loads, print orientation, and BOM.

Print each functional part in its own orientation on **one** plate. Rollers print **standing** (axis Z); they are not a PIP cluster. Save the assembled `.nbcad`, then `solid_move_copy` parts onto the bed and `solid_export_3mf` once as `01-kit`. Appearances: PLA Orange (stator, rollers, retainer) and PLA Glow (rotor). Do not export the assembled nest as the print job.

Minimum wall 1.6 mm (4 nozzles). Functional holes are complete XY circles. Disable grid snap. Prefer locked circles. Draw airfoil polylines with **ctrl held** or ortho-snap will square the section into a plate.

## Build (keep the stand, cut unused plastic)

1. prompts/get model_print_kit. cad_list_all_tools. **cad_new_project on a blank document.** `solid_scene` must show **0 bodies** before the first extrude. Do not continue a recovered or older Print Kit Tutor (tan nest, red disc, leftover helix planes). Desktop: File → New, then build — or File → Open the current `Print-Kit-Tutor.nbcad` after this exam writes it. cad_set_document_name Print Kit Tutor
2. Stator: **Y-frame** + lower race + open top-load fence + short D-journal + snap bead. One piece, print flat. No separate axle disk
3. Rotor: **root plate** (print sit + upper thrust race, ≥5 mm). Loft three helical **NACA 0021** from the plate top (sit plane), open drafted tips. Airfoil through the plate — no rectangular arms. Do not start the loft from a surface above the plate. Do not grow a tall drum or moment webs on the plate
4. **Radial-axis** rollers on a large PCD **under the blade roots**. Top-load slots, not PIP. Fence ID looser than the plate bore. Clocked C-snap retainer on the journal shoulder
5. cad_set_workspace assembly. One `assembly_create_component` per moving body (stator, rotor, each roller, retainer — no extra occurrence). Ground the stator. Revolute rotor_spin about Z; each roller about its radial axis (not a spar); rigid retainer_sit. cad_set_focus drawing. Sheet + auto-layout + notes
6. cad_set_focus print. set_body_appearance to **PLA Orange** and **PLA Glow** only. solid_export_preflight. Save the assembled `.nbcad`. **Delete** any prior `Print-Kit-Tutor/` 3MFs (and `Print-Kit-Tutor.3mf`). `solid_move_copy` the parts onto one bed (rotor standing on the root plate, rollers standing, others flat). Then `solid_export_3mf` **once** as `01-kit` with every kit body_id. The folder must contain exactly that file.
7. `cad_set_project_visibility`: hide every construction plane (`hidden_datum_plane_ids`) and finished loft sketches (`hidden_sketch_names`). The shipped `.nbcad` must read as the merged-stator kit, not orange datum stacks.
8. Write the design report. Include role-based fits, scale vs X2D, roller PCD, why the pack is a thin flat thrust (not a washer, not a tall drum), why two flats must merge, and why a rubbing retainer fails

## Design report (required deliverable)

Write `%USERPROFILE%/Documents/noBS-CAD/Print-Kit-Tutor-design.md` (and the JSON report beside the project). List the one laid-out `01-kit.3mf`. The exam fails if this is missing, empty, or cost-free, or if retired plates are still on disk.

The report must include:

### 1. Iteration log (what failed, why)

At least the real product faults: scatter, colliding spinner, helical C, hoop sector, turntable, flat plate, straight NACA in a fat cage, uniform +0.40 on every hole, PIP a lying roller, bed-printed friction bore with no lead-in, tenoned separate wings, tall skinny shaft, two-land sleeve that cannot take tip moment, loose bushing sandwich with unmatched heights and no attach, washer/pancake stack, tall drum / can on a cracker, standing-Z pucks, tangent-axis rollers, rectangular print arms, inboard pack / cage as journal, separate axle disk + cage disk, rubbing retainer washer.

### 2. Design process

- Architecture (H-Darrieus, short center, why not a tall mast, why not a tall drum)
- Airfoil citation (2026 sources + the section you actually built)
- Solidity, estimated TSR, estimated Re
- Fit table (running / slip / friction)
- Thin flat thrust vs tall drum vs two-land sleeve vs 608 vs loose bushing
- Scale parameter vs X2D envelope
- Service-finish plan and print orientation
- Assembly drawing

### 3. Final product

BOM of the stator / rotor / rollers / retainer, stack, assembly order, how the plate is the upper thrust race and how the blades grow from that plate.

### 4. Printing cost — plastic and material

CAD solid volume, estimated print mass (PLA Orange + PLA Glow, 1.24 g/cm³ × print-volume factor 0.42), filament cost at the spec $/kg. Do not invent a $0 kit.

## Grade

Timeline ok. Individual parts, then an assembly. One-piece helical rotor. Role-based fits. Printed thin flat thrust under the plate (no 608, no tall drum, no loose bushing). Parts that should print flat, do. Scale fits an X2D at 1.0. Assembly drawing with notes. 3MF is a PK zip per plate. **Design report with iteration, girth check, and cost exists.**
