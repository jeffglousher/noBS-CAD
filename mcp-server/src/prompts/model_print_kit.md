CAD synthesis tutor — design a fully printed, omnidirectional VAWT that would actually run, then grade FDM tolerancing.

Spec: scripts/fixtures/print-kit-tutor.spec.json (id fdm-print-vawt).
Contract: docs/agentic/PRINT_KIT_DESIGN.md.
Rerun: npm run test:mcp-print-kit.
Nozzle = {nozzle} mm. Every printed-to-printed running or slip fit is +{nozzle} mm diametral in CAD. No FDM press fits. Slicer XY hole compensation stays 0.

This prompt is adversarial. A kit that spins in the grader and fails as a turbine is a FAIL. Read the reject list before you sketch.

## Reject list (automatic fail)

Do not ship any of these. They have already been built. They are not turbines.

- Print-bed scatter / coupon / “kit of parts” that does not assemble on one axis
- Assembled spinner whose rotor collides with the posts
- Leftover helical C-loft buckets
- Concentric ring sector / hoop / “scoop” whose concave face points at the axis (no net torque)
- Flat rectangular plate sold as a wing (a plate is a vane, not an airfoil)
- Turntable, lazy Susan, paint wells, or any platter that throws the wing away
- One-sided vane, even blade count that cog-locks, or a rotor that only works in one wind azimuth
- Metal 608 / catalog roller / ball bearings as hidden parts. Prefer printed bushings. Rollers are a later table, not this exam
- Press fits, same-angle lifted cones (parallel surfaces never touch), hub swallowing the shoulder
- A report that only says READY TO PRINT
- A fat frame: plate or base OD > 1.55 × rotor tip diameter (the Ø90 cookie around 12 mm slats)
- Posts thicker than 0.40 × chord
- Straight prismatic airfoils (a fence). Helix is required

If the solid looks like a broken rim, a fence slat, or a lid, start over.

## Product

A small **directionless vertical-axis wind turbine** (VAWT). Wind azimuth does not matter. There is no yaw. The machine is a two-bearing stand plus a rotor the stand can hold.

Architecture (required): **helical / Gorlov H-Darrieus** with a **symmetric airfoil** section. Symmetric because the blade sees reversing α every revolution. Helical because a straight extrusion is idle for most of the rev and has hard torque ripple — at desk Re that kills self-start. Directionless means: odd blade count, 120° spacing, chord tangent to the cylinder at every station, identical blades, no preferred wind azimuth.

Helix rule: loft a **closed NACA** at ≥2 stations. Mid-chord stays on radius R. Chord stays tangent. Twist ≥45° over the span (spec: 60°). Root at 30° so the 60° helix stays in the post bays. Do not rotate the section in place (that grows tip radius into the posts). Do not fake twist with a leftover C.

Optional (do not substitute for the airfoil): a small inner drag starter (Savonius cup or 2026 adaptive Darrieus–Savonius flap, Gu & MacDonald & Tang, *Flow* 2026). If you add one, it must clear the airfoils and the posts and still assemble with no hardware.

## 2026 airfoil (required)

Cite a real section. Build that section. Do not invent a “sort of wing.”

2026 VAWT dynamic-stall work (Tirandaz / Rezaeiha line; *Energies* 19(7) 1615, 2026) says **coupled** thickness, thickness-position, and leading-edge radius matter. The favorable low-TSR band is:

- t/c = **21–24%** (thick enough to survive reversing α and to print)
- xt/c = **27.5–35%** (aft of a skinny NACA 0012)
- reduced LE radius index I ≈ **4.5** (not the default I = 6.0)
- Best reported stand-in in that study: **NACA 0024–4.5/3.5** vs a NACA 0018–6.0/3.0 baseline

Range-wide Darrieus optimization (2026, Joukowsky / MOP) still beats NACA 0021 across TSR. For this desk-scale print, the printable required section is **NACA 0021** (t/c = 0.21) with a **blunt trailing edge ≥ 2 nozzles** ({nozzle} × 2 mm). If you can construct the modified 0024–4.5/3.5, do. If you cannot, NACA 0021 with the blunt TE is the honest stand-in — not a rectangle of the same bounding box.

Low-Re flap papers (plain / Gurney / hybrid, 2026) are optional add-ons. Do not use a flap to excuse a flat plate.

## Grouped expected design (build this, not a remix)

The rotor is the product. The frame is a stand. Size the stand to the rotor.

| Group | Spec (do not “improve” by fattening the cage) |
|-------|-----------------------------------------------|
| Section | NACA 0021, t/c 0.21, TE ≥ 0.8 mm, 8 stations, ctrl held |
| Rotor | N=3, c=16 mm, R=24 mm, span=48 mm, σ≈0.32, 60° helix, 3 loft stations |
| Bay | Root azimuth 30°/150°/270°. Helix +60° so mid-span is mid-bay. Tip ~R25.3 |
| Stand | Base/plate **Ø72 × 5 / 6**, not Ø90. Posts **Ø5 on R32** (inner R29.5). ≥4 mm air to tip |
| Girth gates | plate/rotor D ≤ 1.55; post/chord ≤ 0.40; span/chord ≥ 2.5 |
| Fits | +{nozzle} mm every running/slip. Printed cup+land + printed sleeve. No 608 |
| Stack | plate_z = 67 (5+10+2+48+2). Hub sits on the shoulder. Tenon at the **root** station |

Sanity: if the plates look like cookies and the blades look like garnish, the girth gates failed.

Desk Re (c=16 mm, 3–5 m/s, TSR ~2) ≈ 7e3–1.4e4. Thick symmetric is still the right family.

## Service finish (dynamics)

FDM layer lines are roughness. Unfinished printed airfoils pay Cd. Taiwan small-turbine work and 2024–2026 ABS airfoil finish studies: vapor-smoothed ABS gained ~27% L/D; Ra can drop ~80% with acetone vapor vs as-printed. That is a **service finish**, not hardware.

Specify, in the report, all of:

1. **Print orientation per body** so aero skins are not stair-stepped chordwise. Blades: layer lines **parallel to span** (print standing on the TE blunt, or on the face and iron the skin). Journals: bushing as a ring on the bed (bore is an XY circle). Shaft on the land, not on the cone tip
2. **As-printed Ra class**: assume 10–25 µm as-printed FDM. Target service Ra ≤ 5 µm on blade skins and journal bores
3. **Finish process (no extra machine parts)**: PLA — sand 400 → 800 → 1000 on blade skins, optional filler primer (consumable). ABS — acetone vapor 40–50 min class; do not immerse (cracks, dimensional loss). Do not vapor-smooth running fits until you re-measure clearance
4. **Forbidden on aero / journal faces**: fuzzy skin, fuzzy supports, ironing skipped, elephant-foot left on the land
5. **Trailing edge**: blunt ≥ 2 nozzles. A knife-edge TE that the slicer drops is not an airfoil

## Rotation — bushings, not bearings

Prefer **printed bushings** over roller / ball bearings at this size. No 608, no 623, no thrust ball, no metal pin.

Required running set (two-bearing stand):

- **Thrust**: 45° cup in the base (female r5) + smaller male cone (r4.8) + Ø13 × 0.8 annular land with 0.20 float. Relief Ø3 at the apex. A lifted same-angle cone is not a bearing
- **Lower radial**: the land + cup center the shaft. Do not add a hidden roller under the hub
- **Upper radial**: printed sleeve Ø8.4 / Ø14 × 4 in a shouldered seat (Ø14.4 × 4, 2 mm land). L/D of the sleeve is 0.5 — the exam minimum. If you lengthen the sleeve, thicken the plate so the land remains
- **Drive**: double-D 6.0 / 6.4 in the hub zone only
- **Wing mount**: socket 8 × 6 × 5, tenon 7.6 × 4.8 (+{nozzle} / 0.20 axial float). The wing **drops** in. It is not a press and not a screw
- **Retain**: cap Ø20 × 2.4, 0.20 float. No fasteners

Clearance is a design input: +{nozzle} mm diametral on every printed-to-printed running or slip interface, including the tenon. PLA-on-PLA is a demo spin, not a 1000 h bearing. Say that in the report. PETG / nylon bushings are a later material swap, not a reason to import a 608.

## Print and assemble — no additional hardware

No screws, nuts, heat-set inserts, metal shafts, metal bearings, glue as a fit, or rubber bands. The stack is the assembly:

base → shaft → hub → three wings → top plate → bushing → cap

noBS CAD has no mates, instances, or configs. Place bodies by construction. That gap is real (docs/agentic/ASSEMBLY.md).

Print each body in its own orientation. The exam shows the assembled park so the mechanism is readable. The project file cannot store a second print layout — the report must list print orientation.

Minimum wall 1.6 mm (4 nozzles). Functional holes are complete XY circles. Disable grid snap. Prefer locked circles. Draw airfoil, socket, and tenon polylines with **ctrl held** or ortho-snap will square the section into a plate.

## Frame (keep the stand, shrink the girth)

The two-bearing idea stays. The Ø90 cookie does not.

1. prompts/get model_print_kit. cad_list_all_tools. cad_new_project. cad_set_document_name Print Kit Tutor
2. Base Ø72 × 5. Cut the 45° cup. Join three Ø5 posts on R32 at 120° that continue **through** the top plate and stand 2 mm proud
3. Shaft: revolve on XZ. Male cone r4.8, land, Ø8 journal, Ø16 shoulder, double-D, upper journal through the plate
4. Hub Ø28 × 8 **sits on** the shoulder. Ø8.4 bore, double-D 6.4, three sockets at the **root** azimuths (30°/150°/270°)
5. Each wing: loft **NACA 0021** at 3 stations, 60° helix on the R24 cylinder, chord tangent, blunt TE, tenon into the root socket. Ø22 windows at mid-helix
6. Top plate Ø72 × 6 at z=67. Post holes that locate. Bushing seat with a land. Sleeve. Cap
7. cad_set_focus print. set_body_appearance. solid_export_preflight. solid_export_3mf slicer_target=bambu_studio
8. Write the design report. Include the girth ratios and why helix. A green export with a fat frame or a straight fence is still a fail

## Design report (required deliverable)

Write `%USERPROFILE%/Documents/noBS-CAD/Print-Kit-Tutor-design.md` (and the JSON report beside the 3MF). The exam fails if this is missing, empty, or cost-free.

The report must include:

### 1. Iteration log (what failed, why)

At least the real product faults: scatter, colliding spinner, helical C, hoop sector, turntable, flat plate, straight NACA in a fat Ø90 cage. For each: what it looked like, why it cannot make directionless torque or why the frame out-girths the rotor, what you changed.

### 2. Design process

- Architecture choice (H-Darrieus, why not Savonius-only, why not HAWT)
- Airfoil citation (2026 sources + the section you actually built)
- Solidity, estimated TSR, estimated Re, tip clearance
- Fit table (every running/slip interface, +{nozzle} mm)
- Bushing vs bearing decision
- Service-finish plan and print orientation
- What you would change at a larger scale (catalog bearings, higher AR, modified 0024–4.5/3.5, optional hybrid starter)

### 3. Final product

BOM of the nine bodies, stack heights, assembly order, how the wing uses the frame.

### 4. Printing cost — plastic and material

For each body and the kit total:

- CAD solid volume (mm³ and cm³)
- Estimated print mass at the spec filament (Bambu PLA Basic, 1.24 g/cm³) × print-volume factor 0.42 (3 walls + ~15% gyroid on bulky parts; blades closer to solid — say so)
- Filament cost at the spec $/kg (default $20/kg PLA)
- Note: electricity and time are optional; plastic + material is required

Do not invent a $0 kit. Do not omit the blades.

## Grade

Timeline ok. ≥9 coaxial bodies. Posts through plate. Wings in hub sockets. Cone/land thrust. Double-D drive. Even 3+3. **Named helical NACA, not a plate, hoop, or straight fence.** Frame/rotor ≤ 1.55. Post/chord ≤ 0.40. Printed bushings. 3MF is a PK zip. **Design report with iteration, girth check, and cost exists.**
