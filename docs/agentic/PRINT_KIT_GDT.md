# Print-kit tutor — GD&T and printability study

Pro-forma study for benchmark #1 (`fdm-print-vawt`). The first assembled
spinner looked like a kit and was not one. The next kit was a leftover
helical loft inside a good frame. Replacing that with a turntable threw
the frame away — and left the wing with nothing to use. This page is the
correction record. Numbers live in `scripts/fixtures/print-kit-tutor.spec.json`.

**Current machine (2026-08-17):** one-piece helical rotor on a **thin**
root plate with **organic airfoil roots** (appearance reference) and a
short **tip taper to a flat landing**; section is **NACA 0024-4.5/3.5**;
**one stator** (thin Y-frame + race **ring** + **keeper
walls** + open top-load fence + short D-journal); **thin flat thrust
under the blade roots** (8 radial-axis rollers, pack height = roller Ø);
fence sits on the race ring (ID looser than the plate bore); clocked
C-clip snaps into the journal groove and does not rub the rotor.
Clip and pack integration research (why the printed rollers slide,
why the C-clip is awful): [PRINT_KIT_BEARING.md](PRINT_KIT_BEARING.md).
The two-bearing post / tenon / cone stand in the sections below is
**historical** — do not rebuild it.

## Product fault (why the turntable was wrong)

The Ø90 post-and-plate stack is a **two-bearing stand**: cup in the base,
sleeve in the top plate, posts locating the upper bearing. That is how a
small VAWT is built. The wing uses that stand by mounting to a hub on the
shaft and sweeping the bays between the posts.

A lazy-Susan platter with paint wells does not use the frame. Helical
C-buckets joined to a hub also failed: they were leftover loft, not a
mount. A concentric ring sector is also not a wing: the concave face
points at the axis and cannot catch wind. The wing is a **vertical
blade** (chord tangential, span up the bay) with a tenon into a hub
socket. The stand is a Y-frame — open bays, no cookie plate.

## Faults found (and closed)

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Product (turntable) | Competent bearings, no wing. The frame had nothing to do. | **Printed VAWT.** Keep the two-bearing frame. Hub sockets. Three wings that drop on. |
| Product (leftover loft) | Two helical C-buckets joined to a hub. Not a mount, not assemblable as a wing. | Separate wing bodies. Tenon in socket. No loft. |
| Product (hoop sector) | “Scoop” was an 80° annulus at r20–r28. Concave faces the axis. Looks like a broken rim. Cannot make torque. | Named symmetric airfoil, chord tangential. |
| Product (flat plate) | 12 × 2.4 × 32 rectangle. A vane, not a section. Directionless VAWT sees reversing α. | **NACA 0021** (t/c 0.21), blunt TE ≥ 0.8 mm. 2026 band is t/c 21–24%. |
| Rotor vs posts | C-buckets sweep ~Ø56. Posts on R24 × Ø6 occupy R21–27. They collide. | Posts **Ø5 on R32**. Inner wall R29.5. Helical tip ~**R25.3**. ≥4 mm air. Root at 30° so the 60° helix stays in the bay. |
| Posts vs plate | Posts stopped at the plate bottom. Holes did not locate. | Posts continue **through** the 6 mm plate and stand **2 mm proud**. |
| Bushing seat | Plate and seat were both 4 mm. Seat was a through-hole. Bushing falls. | Hanging boss. Seat **8 mm** from the top. **2 mm land** under a two-land sleeve. |
| Short full sleeve | L/D 0.5 rubbed a full Ø8.4 cylinder. Extra drag, little alignment. | Sleeve **8 mm** (L/D **1.0**), relief Ø9.2, two 1.6 mm lands. Fits stay **+0.40**. |
| Cookie plate | Ø72 / Ø90 disc around three posts. Plastic that is not a bearing or a rib. | **Y-frame**: hub + ribs + pads. Envelope still ~Ø74. |
| Cone “seat” | Same 45° + tip lift = parallel cones. No contact. Shaft falls. | Female 45° r5. Male r**4.8** (0.20 radial). Thrust on a **Ø12 × 0.8 land** with **0.20 float**. |
| Rotor drive | Round-on-round slip. Rotor need not turn the shaft. | **Double-D** 6.0 / 6.4 in the hub zone only. |
| Wing mount | No interface. Blade was a leftover solid. | Hub **sockets** 8 × 6 × 5. Wing **tenon** 7.6 × 4.8 (+0.40 / 0.20 float). |
| Hub vs shoulder | Hub started at the shoulder plane with an Ø8.4 bore around the Ø16 shoulder. They occupy the same volume. | Hub **sits on the shoulder top**. `plate_z` includes the 2 mm shoulder. |
| Twisted buckets | Radial C-walls ortho-snapped. Loft stations had no closed profile. | No loft. Vertical plates. Socket/tenon lines drawn with **ctrl held**. |

## Datum scheme

| Datum | Feature | Role |
|-------|---------|------|
| **A** | Base bottom | Primary. Frame print bed. |
| **B** | Journal / cone axis | Secondary. Turbine axis. |
| **C** | 3× Ø5 posts, Ø64 PCD, 120° | Tertiary. Top-plate location. Wings sit in the bays. |

Position of C to B: **Ø0.4 MMC** (one 0.4 mm nozzle). Tighter than that is a
gauge-print problem, not a CAD problem.

## Fits (modeled in CAD, slicer XY hole comp = 0)

| Joint | Class | CAD |
|-------|-------|-----|
| Journal ↔ cup ID | Running | Ø8.0 / Ø8.4 (+0.40 diametral) |
| Posts ↔ plate | Location slip | Ø8.0 / Ø8.4, through |
| Bushing ↔ seat | Location slip | Ø14.0 / Ø14.4 + 2 mm land |
| Double-D drive | Location slip | 6.0 / 6.4 across flats |
| Wing tenon ↔ hub socket | Location slip | 7.6 / 8.0 width; 0.20 axial float |
| Cone center | Clearance | 0.20 radial at the mouth |
| Thrust land, cap | Axial play | 0.20 float — **no coincident running faces** |

Shoulder ↔ hub bottom is a **sitting** face (they rotate together). That
contact is intentional.

## Printability (0.4 mm Bambu, 0.20 mm layer)

| Body | Print orientation | Notes |
|------|-------------------|-------|
| Base | As assembled (A on the bed) | 45° cup needs no support. Posts print up. |
| Shaft | Land / shoulder on the bed | Do not print on the cone tip. Double-D flats are vertical walls. |
| Hub | Face on the bed | Sockets are complete XY pockets. |
| Wing | Standing, layer lines spanwise | NACA 0021. Blunt TE ≥ 0.8 mm. Sand skins. |
| Top plate | Flat | All functional holes are complete XY circles. |
| Bushing | Ring on the bed | Bore is an XY circle. |
| Cap | Flat | Same. |

Minimum wall 1.6 mm. No horizontal holes. No FDM press fits. Break 0.4×45
on post tips in the slicer if they hang on the plate.

## Assembly (proper)

1. Shaft cone into the cup — land floats 0.20 above A.
2. Hub double-D onto the journal; hub **sits on** the shoulder.
3. Each wing **drops** its tenon into a hub socket. The blade stands in a post bay.
4. Top plate down the three posts (holes actually engage).
5. Sleeve into the shouldered seat.
6. Cap onto the round journal, 0.20 above the plate.

If those steps are not visible in the solid, the exam failed.

## Wings (hub off the lid)

The top plate used to hide the rotor. These views are hub + three wings
only. The section is **NACA 0021**, not a hoop and not a flat plate.

![Wings on the hub, isometric](images/wings-iso.png)

![Wings on the hub, top](images/wings-top.png)

![One wing, helical NACA 0021 and tenon](images/wings-one.png)

## Assembled park

![Assembled park, isometric](images/assembled-park-iso.png)

![Assembled park, front](images/assembled-park-front.png)

![Assembled park, top](images/assembled-park-top.png)

## As-built stack (spec)

| Body | z min–max | Notes |
|------|-----------|--------|
| Base + posts | 0–73 | Y-frame. Ø5 posts through the 67–71 arms, 2 mm proud |
| Shaft | 0.5–75 | Narrow land above the cup; journal through the cap |
| Hub | 17–25 | Sits on the shoulder; sockets from the top |
| Wings | 17–65 | Helical NACA 0021, 60° twist; tenon at the root; 2 mm air to the plate |
| Top plate | 61–71 | Y-frame 67–71 + hanging boss 61–67 |
| Bushing | 63–71 | Two-land sleeve on the 2 mm land |
| Cap | 71.2–72.8 | 0.20 float |

Blade tip ~R25.3 < post inner R29.5. Envelope/rotor ~1.46. Nine coaxial bodies.

## 2026-08-16 assembly pass (closed)

The two-bearing nest above is the previous exam. It printed and it
graded, but it was the wrong product for additive manufacturing:

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Uniform +0.40 | Every hole used the running clearance. Locates were sloppy; nothing that should be tight was. | Role-based: running +0.40, slip +0.28, friction +0.16. Slicer XY hole comp stays 0. |
| Tenoned wings | Three separate blades + hub is a puzzle, not a rotor. | Hub + 3 helical NACAs are **one body**. Open drafted tips. |
| Tall skinny shaft | Support tower; weak in bending at the blade tips. | Short square stator post. Axle is a flanged inner-race **puck**, printed on the flange. |
| Two-land sleeve only | L/D 1.0 is a journal, not a moment bearing. | Large-PCD thin flat thrust under the plate (radial-axis rollers, min Ø8, pack height = Ø). |
| No CAD assembly | Multi-body nest, no components/joints, no drawing. | 5 components, grounded base, revolute, A3 sheet with notes. |
| No scale | Desk-size only; not an X2D-max source. | Spec numbers are X2D-max (256×256×260, 8 mm margin). `scale` 0.4 in the exam. |

### Fits (current)

| Joint | Class | CAD |
|-------|-------|-----|
| Rollers ↔ races (assembled) | Running | +0.40 diametral |
| Rollers ↔ cage pockets | Running | +0.40 diametral. Rollers print standing and drop into windows on the flange. Do not PIP a lying roller. Cage is not a pickup cartridge. |
| Retainer ↔ square post | Slip | +0.28 |
| Axle square bore ↔ post | Friction locate | +0.16 |
| Plate bore ↔ inner race | Running | +0.40. This is the radial land. The plate is not keyed to the post. |
| Cage ID ↔ journal | Clearance | Cage ID = plate bore + 2 walls. Spacer, not a tighter journal. |
| Plate / cage ↔ flange land | Thrust | 0.20 axial float; pack sits on the flange, plate sits on the pack |
| Rollers ↔ plate underside | Thrust | Upper race is the plate. Cage height = roller height. |
| Plate ↔ retainer washer | Thrust | 0.20 axial float; retainer covers the plate bore, OD between bore and flange |
| Axle ↔ base | Sitting stator | Coincident land. They do not spin relative to each other. |
| Plate bore ↔ short journal | Running | **Not a friction fit.** Friction here locks the bearing. |

### Print (current)

| Part | Orientation |
|------|-------------|
| Base | Flat (Y-frame + post) |
| Axle | On the flange |
| Rotor | Standing on the root plate, tips up. Blade bottoms are the sit-plane cut. |
| Cage | Flat |
| Rollers | Standing (axis Z); assemble lying down (axis radial) |
| Retainer | Flat |

### Assembly (current)

1. Axle puck onto the square post (friction locate). Flange **sits** on the base.
2. Drop the cage onto the flange. Cage height equals roller Ø. Cage ID is looser than the plate bore.
3. Drop each roller into a window (they sit on the flange; the cage only spaces them). Each roller revolute is about its radial axis. Do not pick the cage up as a preloaded cartridge — the windows are open so the rollers can touch both races.
4. Drop the rotor on the pack: plate bore over the short journal, plate underside 0.20 above the rollers. The pack outer land is under the blade roots.
5. Retainer washer **square-slip** on the post, floats 0.20 above the plate.

If those steps are not visible in the solid, the exam failed.

### 2026-08-16 fit / plate pass (closed)

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Coincident running faces | Hub sat on the flange; retainer sat on the hub. Zero axial play looks fused and binds in FDM. | 0.20 float at every running land. Inner race grows by the same float so rollers stay on the race. |
| Retainer cap | Retainer OD covered the hub, so the washer looked like a disc through the rotor. | Washer OD is between hub bore and hub OD. |
| Five colors / five plates | Assembled nest exported per part; materials did not match the two-filament print. | One laid-out `01-kit.3mf`. PLA Orange + PLA Glow only. |

### 2026-08-16 bushing / mount pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Hub as the outer race | Cage stuffed inside the hub wall. No housing, no shoulder, no “wheel on a bearing.” | First correction was a distinct bushing. That still had no attach path — see the cup pass. |

### 2026-08-16 one-frame cup pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Loose bushing sandwich | Separate orange ring, postage-stamp flange, cage taller than the rollers, blades not attached to the race. Unmatched heights. No way to take a heavy overhung load. | Rotor **is** the frame: plate = thrust floor, cup ID = outer race (height = roller + float), blades grow from that plate. Drop rotor, then cartridge, then retainer. |

### 2026-08-16 washer-cup / pancake pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Washer cup | Matching an 8 mm land to 8 mm rollers produced another stack of flats. From a 3/4 view the green wall disappeared and the orange cartridge sat on the plate. | Height-matching flats is not a bearing. The next wrong answer was a tall drum — see the thin-thrust pass. |
| Can on a cracker | Cup got height; blades still only met a 5 mm plate. Overturning moment bent the disk and never reached the race. | The next wrong answer was webs climbing that wall. Overturning is a couple across a large PCD, not a taller can. |

### 2026-08-16 thin thrust pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Tall drum | A 28 mm orange tower with rollers filling the ID is a journal you can see from the side. The machine's job is rotation about Z. | **Thin flat thrust** under the plate: flange = lower race, plate underside = upper race, radial-axis rollers on a large PCD. Short journal centers only. |
| Moment webs | Blade roots climbing the cup wall doubled down on the tall system. | Delete the drum and the webs. Width takes the tip moment. |

### 2026-08-16 additive / FDM pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| PIP at assembled running | Cage pockets at +0.40 are 0.20 mm/side. Same-plate first layers weld. | PIP pockets +0.80 (2 nozzles). Assembled races stay +0.40. |
| Bed-printed friction bore | Plate bore and axle square sit on the bed. Elephant foot closes +0.16. | 0.80 mm lead-in on plate bore, axle square, retainer square. Functional land starts above layer 1. |
| Race nested around rollers | Race-to-roller at +0.40 is 0.10 mm/side on the same plate. They fuse. | Cup is integral to the rotor. Cage+rollers are the PIP cluster dropped in after. Axial capture is flange + retainer, not a top lip. |

### 2026-08-16 blade root / deck pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Overhung blade roots | Loft started mid-hub (`hub_h × 0.35`), out at wing radius. First layers of a standing print were air. | Root plate is the sit plane. Airfoil through the plate — no rectangular arms. Helical loft starts at `blade_root_z` = plate top. |
| Tiny ring + blades from the surface above | Ø41 deck + skinny arms. Airfoils started from the arm top. No housing, no install path. | Root plate out to the blades. Loft from plate top so the draft ends on that flat. |

### 2026-08-17 radial-axis thrust pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Standing-Z pucks | Ø8 × h3.2 pucks spin about Z. End faces slide on flange and plate. That is not rolling under −Z. | Cylinders, **axis radial**. Pack height = roller Ø. Print standing, assemble lying down. |
| Tangent-axis rollers | A tangent axis rolls inward/outward. | Relative motion at the race is circumferential, so ω × (±Z) needs e_r. |
| Base boss larger than the plate | Ø84 orange halo under a Ø76 plate. | Boss ≤ flange, always smaller than the plate OD. |
| Rectangular print arms | Blade roots were arms + spars + stumps. | Airfoil through the plate. |

### 2026-08-17 load-path pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Inboard pack | PCD at 58% of the plate (exam 44 mm vs blade R 34). Blade roots cantilevered on 5 mm PLA. The couple never reached the race — same cracker as the tall drum, just thinner. | PCD out so the roller outer land reaches ~0.9× blade R. Flange sized to the race and stays inside the plate (no `cage_od+4` halo). |
| Cage as journal | Cage ID was journal + slip (tighter than the plate bore). The spacer stole the radial land and rubbed. | Cage ID = plate bore + 2 walls. Plate bore is the radial running land. |
| Boss = race OD | Base boss tracked the flange, so a wide pack reprinted a solid orange cylinder under the plate. | Boss only seats the axle. Race overhangs the boss by ≥8 mm. |
| Pickup cartridge | Pocket Ø > cage height, so windows are open top and bottom (required for race contact). Rollers fall out if you lift the cage. | Assemble on the flange: cage, then rollers into the windows, then the rotor. |

### 2026-08-17 stator merge / top-load / C-snap pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Separate axle disk + cage disk | Two flats on the plate. Extra plastic, extra assembly, and a square washer that rubs the rotor. | **One stator:** Y-frame + lower race + open fence + journal. The race *is* the Y-frame center. |
| PIP lying rollers | OD prints as layers. Dimensions are poor. | Print rollers **standing**. **Top-load** slots (running + 2 nozzles) so they drop in from above with no support. |
| Rubbing retainer washer | Square-slip washer sat on / near the plate and added friction on the turbine. | Clocked **C-snap** (D-hole + C-gap) sits on the journal shoulder, 0.20 above the plate. Pull to remove. Only one orientation. |
| Sliding contacts on the turbine | Plate bore as friction, cage as journal, retainer as a running face. | Only rolling contacts: rollers ↔ races. Plate bore is running. Fence ID is a spacer. Retainer is not a running face. PLA-on-PLA is a demo; service dry PTFE on the races. |
| Cookie race under the Y-frame | Merging the axle disk into a Ø74 solid reprinted the plastic we dropped. Rollers only contact a ring. | Race is a **ring** (ID ≈ PCD − roller length). Fence sits on that ring. Y-frame hub + ribs + pads stay open. |
| Hourglass journal | Fat shoulder + snap bead above a thinner neck. Plate bore cannot pass the fat top, so the rotor will not drop on. | Journal is a **constant pass Ø** smaller than the plate bore. Clocked **C-clip** snaps into an undercut groove above the plate. Pull the C-gap to remove. D-flat is only on the tip, not through the running land. |

### 2026-08-17 capture / thin / organic pass

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Rollers slide out | `race_id` sat at the roller inner end. The top-load pocket is longer than the roller by running, so the cut punched through the race ID into the open Y-frame. | `race_id = PCD − roller length − 2×keeper`. Inner and outer end walls survive the cut (`keeper > running/2`). Still top-load — no bars over the pack. Axial capture stays the plate. |
| Thick stator | Y-frame / race floor was 6 mm at exam scale. Extra plastic, not extra stiffness where it matters. | Base floor 3.2 mm (exam **4.8**). Race stays a ring. |
| Thick plate / sharp join | Plate floor was 5 mm. A hard airfoil cut on a thinner plate is a crack starter and a print cliff. | Plate floor 3.2 mm (exam **4.0**). First loft station above the plate is a fatter airfoil (`root_scale` 1.38 over `root_blend_h` 6.0). Through-plate stump stays the sit-plane chord. |
| Blunt tip | Square-cut open tip is a dirty aero edge. | Last loft section stays planar (flat landing). Add a short chord taper (`tip_scale` 0.72 over `tip_taper_h` 4.0) into that face. |

## Clip study (unlocked for the bearing-integration pass)

The shipped retainer is a **printed-flat C-washer**: D-hole, rectangular
C-gap, seats in a 2-nozzle undercut on a **constant-pass** journal.
Exam-scale numbers (scale 0.4, 0.4 mm nozzle):

| Feature | mm | Role |
|---------|---:|------|
| Journal / pass Ø | 12.0 | Plate must drop over this. Nothing fatter. |
| Plate bore | 12.4 | Running +0.40 |
| Groove Ø | 10.4 | 2-nozzle undercut |
| Clip ID (D-hole) | 10.68 | Slip +0.28 in the groove |
| Clip OD | ~20.4 | Washer rim |
| Clip thickness (radial) | ~4.9 | `(OD − ID) / 2` |
| Clip height | 2.0 | Groove + 0.20 float |
| C-gap | 3.2 | Flex slot — **not** a side-entry |

That is an **annular stretch ring**, not a cantilever and not an
E-clip. Assembly is **axial**: the ID must open from 10.68 to 12.0
(ΔD 1.32 mm, 0.66 mm radial) to pass the tip, then contract into the
groove. The C-gap (3.2) is far smaller than the journal (12.0), so the
clip **cannot** slide on from the side. A closed-ring hoop strain of
`1.32 / 10.68 ≈ 12%` is well above PLA's useful snap strain. The gap
turns that into bending at the back of the C, but the section is still
chunky (radial t ≈ 4.9 mm) for only 0.66 mm of opening — a stiff
washer with a slot, not a designed spring.

### What the literature says

| Source | What to take |
|--------|----------------|
| FilamentFeed, *Snap Fit Design for 3D Printing* (June 2026) | PLA allowable strain **2–3%**; PETG 3–5%; nylon 5–8%. Annular snaps **< Ø30 print flat** so stretch is in-plane. PLA fatigue **5–20 cycles**. 3+ walls, 100% infill on thin snaps. Lead-in **30°**; retention **45°** (hand release) or **90°** (permanent). |
| Wevolver / Mandarin3D / UnionFab (2025) | PLA is a one-shot or light-duty snap. Fillet gap/arm roots **r ≥ 0.5 t**. Cantilevers: **L/t ≥ 8–10**, taper thickness 100% → 50% toward the tip. Clearance 0.2–0.5 mm. Flex in XY, never across layers. |
| Formlabs snap-fit note | FDM Z tensile is ~40–60% of XY. A clip that bends out of its layer plane needs a 0.5–0.6 strain knockdown. |
| Bayer / BASF snap-fit handbook (IM, still the strain model) | Annular strain `ε ≈ (D_shaft − D_hole) / D_hole`. Unfilled plastics want assembly strain in the low single digits if the part must come off again. |

Print-flat is already correct for this Ø12 ring. The material lock
(PLA Basic Orange) is also already correct for the kit — just do not
pretend it is a service clip.

### Recommendations for a later pass (do not build now)

1. **Best family for this kit: a printed E-clip / circlip.** Two long
   cantilevers, `L/t ≥ 8–10`, taper toward the tips, fillet roots
   `r ≥ 0.5 t`, **finger tabs** at the mouth. Print flat. Assemble
   **radially** into the existing groove (gap must open to the groove
   Ø, or slightly less with flex). Journal stays a constant pass so
   the rotor still drops on. This is a beam problem (FilamentFeed
   `ε = 1.5 h Y / L²`), not a hoop-stretch problem.
2. **If the C-ring stays:** thin the rim to **2.0–2.4 mm** (5–6 walls),
   add finger tabs at the C-gap, fillet the gap roots, put a
   **30–35° lead-in** on the journal tip, and cut the groove lip at
   **45°** (hand release + a printable overhang). Square 90° groove
   walls are hard to pull off and leave a downward face under the tip
   when the stator prints flat. Keep slip +0.28. Target PLA assembly
   strain ≤ 3%.
3. **Alternate: twist / bayonet tabs on the clip** that flex into the
   groove. Journal still constant-pass. Do **not** put fat lugs on the
   journal — that is the hourglass the plate cannot pass.
4. **Do not:** metal circlips (kit is printed-only), a closed hoop,
   a side-entry gap smaller than the groove, or a clip that rubs the
   rotor. Do not change plastics on this kit.

Service note: print a snap coupon (same groove, same clip section)
before reprinting the whole stator. For a clip that comes off more
than a few times, PETG or nylon is the right later material — not
this exam's PLA orange.

Implementation target is the E-clip in
[PRINT_KIT_BEARING.md](PRINT_KIT_BEARING.md) §6. Do not ship another
stiff C-washer.
