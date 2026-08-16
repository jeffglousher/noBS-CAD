# Print-kit tutor — GD&T and printability study

Pro-forma study for benchmark #1 (`fdm-print-turntable`). The first assembled
spinner looked like a kit and was not one. The next kit was a **super frame
around a leftover VAWT**: posts, lids, and helical C-buckets that graded
clearance while the machine was still not a product. This page is the
correction record. Numbers live in `scripts/fixtures/print-kit-tutor.spec.json`.

## Product fault (why the cage was wrong)

The Ø90 post-and-plate stack taught “posts through plate” and “rotor clears
posts.” Those are assembly-lesson checkboxes. They are not a reason to
print a wind turbine. noBS CAD is local mechanical CAD. Benchmark #1 has
to look like something you would model and print: a **turntable** (lazy
Susan / paint stand). The platter is the part. The foot is smaller. The
keeper is a collar, not a second lid. Helical C-buckets stay out.

Fits and printed bearings from the cage era stay. The machine does not.

## Faults found (and closed)

| Fault | Why it failed | Correction |
|-------|---------------|------------|
| Product | Competent GD&T around two leftover helical buckets inside a Ø90 cage. Looks like a frame, not a machine. | **Printed turntable.** Ø72 platter on a Ø48 foot. Ø28 keeper. Three wells. No posts. No C-buckets. |
| Rotor vs posts | C-buckets sweep ~Ø56. Posts on R24 × Ø6 occupy R21–27. They collide. | Dropped. A turntable does not need a post circle. |
| Posts vs plate | Posts stopped at the plate bottom. Holes did not locate. | Dropped with the cage. Location is the journal + double-D + shoulder. |
| Bushing seat | Plate and seat were both 4 mm. Seat was a through-hole. Bushing falls. | Keeper **6 mm**. Seat **4 mm** from the top. **2 mm land** under the sleeve. |
| Cone “seat” | Same 45° + tip lift = parallel cones. No contact. Shaft falls. | Female 45° r5. Male r**4.8** (0.20 radial). Thrust on a **Ø13 × 0.8 land** with **0.20 float**. |
| Rotor drive | Round-on-round slip. Rotor need not turn the shaft. | **Double-D** 6.0 / 6.4 in the platter zone only. Upper journal stays round for the sleeve. |
| Cap grind | Cap sat on the plate with 0 gap. | **0.20 mm cap float** above the keeper. |
| Hub vs shoulder | Hub started at the shoulder plane with an Ø8.4 bore around the Ø16 shoulder. They occupy the same volume. | Platter **sits on the shoulder top**. `plate_z` includes the 2 mm shoulder. |
| Twisted buckets | Radial C-walls at 10°/20° ortho-snapped (`ctrl_held: false`). Loft stations had no closed profile. | Removed. Even mass is three wells at 120°, not a helical loft leftover. |

## Datum scheme

| Datum | Feature | Role |
|-------|---------|------|
| **A** | Base bottom | Primary. Foot print bed. |
| **B** | Journal / cone axis | Secondary. Turntable axis. |
| **C** | 3× Ø16 wells, Ø44 PCD, 120° | Tertiary. Even platter. |

Position of C to B: **Ø0.4 MMC** (one 0.4 mm nozzle). Tighter than that is a
gauge-print problem, not a CAD problem.

## Fits (modeled in CAD, slicer XY hole comp = 0)

| Joint | Class | CAD |
|-------|-------|-----|
| Journal ↔ bushing | Running | Ø8.0 / Ø8.4 (+0.40 diametral) |
| Journal ↔ platter / keeper | Running / location | Ø8.0 / Ø8.4 |
| Bushing ↔ seat | Location slip | Ø14.0 / Ø14.4 + 2 mm land |
| Double-D drive | Location slip | 6.0 / 6.4 across flats |
| Cone center | Clearance | 0.20 radial at the mouth |
| Thrust land, cap | Axial play | 0.20 float — **no coincident running faces** |

Shoulder ↔ platter bottom is a **sitting** face (they rotate together). That
contact is intentional.

## Printability (0.4 mm Bambu, 0.20 mm layer)

| Body | Print orientation | Notes |
|------|-------------------|-------|
| Base | As assembled (A on the bed) | 45° cup needs no support. |
| Shaft | Land / shoulder on the bed | Do not print on the cone tip. Double-D flats are vertical walls. |
| Platter | Flat on the bed | Rim and wells are complete XY circles. |
| Keeper | Flat | Journal and seat are XY circles. |
| Bushing | Ring on the bed | Bore is an XY circle. |
| Cap | Flat | Same. |

Minimum wall 1.6 mm. No horizontal holes. No FDM press fits.

## Assembly (proper)

1. Shaft cone into the cup — land floats 0.20 above A.
2. Platter double-D onto the journal; platter **sits on** the shoulder (they rotate together).
3. Keeper down the journal (small collar, not a lid).
4. Sleeve into the shouldered seat.
5. Cap onto the round journal, 0.20 above the keeper.

If those five steps are not visible in the solid, the exam failed.

## Assembled park

![Assembled park, isometric](images/assembled-park-iso.png)

![Assembled park, front](images/assembled-park-front.png)

![Assembled park, top](images/assembled-park-top.png)

## As-built stack (spec)

| Body | z min–max | Notes |
|------|-----------|--------|
| Base | 0–6 | Ø48 foot, no posts |
| Shaft | 1.5–34 | Land above the cup; journal through the cap |
| Platter | 16–22 | Sits on the shoulder; 1.5 mm air to the keeper |
| Keeper | 23.5–29.5 | Ø28 × 6 collar |
| Bushing | 25.5–29.5 | On the 2 mm land |
| Cap | 29.7–32.1 | 0.20 float |

Platter Ø72 > foot Ø48. Keeper Ø28 is a collar. Six coaxial bodies.
