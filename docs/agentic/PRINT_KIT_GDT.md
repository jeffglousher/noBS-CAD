# Print-kit tutor — GD&T and printability study

Pro-forma study for benchmark #1 (`fdm-print-vawt`). The first assembled
spinner looked like a kit and was not one. The next kit was a leftover
helical loft inside a good frame. Replacing that with a turntable threw
the frame away — and left the wing with nothing to use. This page is the
correction record. Numbers live in `scripts/fixtures/print-kit-tutor.spec.json`.

**Current machine (2026-08-17):** one-piece helical rotor on a thin
root plate; **one stator** (Y-frame + lower race + open top-load fence +
short D-journal); **thin flat thrust under the blade roots**
(radial-axis rollers, pack height = roller Ø); fence is a spacer (ID
looser than the plate bore); clocked C-snap retainer sits on the journal
shoulder and does not rub the rotor. The two-bearing post / tenon / cone
stand in the sections below is **historical** — do not rebuild it.

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
