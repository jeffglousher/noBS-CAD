# Print-kit bearing — integration research

The aero blades stay ([PRINT_KIT_AERO.md](PRINT_KIT_AERO.md)).
This page is the **thrust-pack + clip** contract for integrating that
rotor. Live files: [PRINT_KIT_FILES.md](PRINT_KIT_FILES.md).
Figures: [figures/print-kit-bearing/](figures/print-kit-bearing/)
(rebuild with `node scripts/print-kit-bearing-figures.mjs`).

Jeff’s printed machine (2026-08-17): **pieces fit, rollers do not
roll, the plate slides on top, it is loud, the clip is awful,
assembly was too hard** (inner keepers / ring partially fixed that
last one). Geometry CAD on this branch is in (U-window, crown,
E-clip). This pass is **roller profile + materials**: hollow
short-land PETG rollers on PLA races, blades stay PLA Glow for a
later generator. Do not touch the airfoil.

## 1. What failed on the printed kit

| Symptom | What that means | Likely CAD cause (confirmed in the compilers) |
|---------|-----------------|-----------------------------------------------|
| Rollers do not turn | No rolling element. The pack is a **slider**. | Slot is a **conforming cylinder** around the OD. |
| Plate slides on top | Traction is sliding friction on a locked roller, or the plate is not on the rollers. | Locked trough + plate on the protruding arc. Fence is lower, so the plate *does* hit the rollers — it just cannot spin them. |
| Loud | Stick-slip of PLA-on-PLA + layer texture. | Dry conforming contact. CoF dry PLA ~0.15–0.4. |
| Hard to assemble | Tight mouth, no funnel, bad clip. | Slot mouth = same Ø as the trough. Clip is a stiff C-washer (12% hoop strain). |
| Missing piece (partially fixed) | Rollers escaped radially. | Keepers / inner ring. Keep that. |

Exam-scale numbers (scale 0.4, 0.4 mm nozzle), from the same helpers
as the tutor (`numbers.json`):

| Feature | mm | Role today |
|---------|---:|------------|
| Roller | Ø8.0 × L11.2 | Right circular cylinder. Print standing. |
| Pack height | 8.0 | `= roller Ø` |
| Fence height | 4.96 | `0.62 × pack` — below the pack, as intended |
| Slot cut | **Ø9.2 × L11.6** | `place_radial_cylinder(top_load_pocket, pocket_len)` **subtracted** from the stator |
| Slot center | 4.0 above race | `z_mid = pack/2` |
| Trough into race | **0.60** | `slotR − z_mid` — the “bottom” is not flat |
| Protrusion above fence | 3.04 | Plate can touch the roller |
| Clip ID → journal | 10.68 → 12.0 | Axial stretch. Hoop strain **12.4%** |

![Current trough](figures/print-kit-bearing/fig-b1-current-trough.svg)

**Fig. B1.** The top-load “opening” and the “bottom” are the same
horizontal cylinder. A roller in a matching cradle is a **journal**,
not a bearing. The sides wrap the generator. The plate slides.

---

## 2. Rolling vs sliding (why a cylinder in a cylinder never rolls)

A rolling element needs **two** things:

1. A load path through a **small contact** (line or ellipse) on the
   races, not a wrapped OD.
2. Freedom to spin: the cage / slot may touch **ends or pads**, not
   the working generator.

Kinematics on two parallel flats, roller radius \(r\), plate speed
\(v\):

\[
v \;=\; \omega r
\qquad\text{(pure rolling)}
\]

If the roller cannot spin, \(\omega=0\) and the contact is sliding
at \(v\). Friction power:

\[
P_\mathrm{slide}
\;=\;
\mu N v
\]

PLA-on-PLA dry \(\mu\sim 0.15\)–\(0.4\)
([Hanon et al. 2023](#ref-hanon-2023); [Srinivasulu 2024](#ref-srinivasulu-2024)).
That is the noise (stick-slip) and the “bearing does nothing.”

A **conforming** trough of nearly the same radius raises the contact
area by an order of magnitude (Hertz → almost a journal). Traction
to *start* rolling has to beat that wrap friction. It loses.

**Required geometry**

| Land | Must be | Must not be |
|------|---------|-------------|
| Bottom race | Flat ring. Cut **stops at the race face**. 0.8 mm chamfer at ID/OD. | A cylindrical cradle. A cookie disk. |
| Opening | Downward **U-window** through the fence. Funnel \(20^\circ\)–\(30^\circ\). Mouth ≥ roller Ø + 1.6 mm. | The same cylinder as the trough. |
| Sides | Clearance ≥ **0.8 mm/side** (2 nozzles). Touch only if the roller walks. | Wrap the OD. |
| Keepers | Inner/outer **end** walls. Already in the kit. | Side walls that pinch the generator. |
| Top race | Plate underside, still flat. Sits on the **crown**, 0.20 modeled float. | Fence rim. Journal shoulder. Clip. |

![Required stack](figures/print-kit-bearing/fig-b2-required-stack.svg)

**Fig. B2.** Flat race, funnel opening, crowned roller, sides do not
wrap. This is the integration target.

---

## 3. “Very small taper” — crown, not a naked cone

Two different tapers. Only one is legal on **parallel flats**.

### 3.1 True tapered roller (Timken / ISO 355)

The generators of the roller **and both races** meet at one point on
the bearing axis (the **apex**). Then every station has
\(v=\omega r(s)\) and there is no kinematic scrub
([Timken catalog](#ref-timken); [ISO 355:2007](#ref-iso355);
[Harris & Kotzalas](#ref-harris)).

![Apex rule](figures/print-kit-bearing/fig-b3-apex-rule.svg)

**Fig. B3.** A cone between two flats **increases** sliding: \(\omega r\)
changes along the length, the flats force one speed.

A printable axial TRB (e.g. [Bombermine 2025](#ref-bombermine)) works
because **the races are cones too**. We would have to taper the
stator race **and** the plate underside, keep the apex on \(Z\), and
accept a steeper stack. That is a later option for combined
overturning + thrust. It is **not** “put a taper on the existing
cylinder.”

### 3.2 What we want on this kit: a small **barrel crown**

Metal cylindrical rollers are **crowned** so Hertz pressure does not
spike at the ends ([Lundberg 1939](#ref-lundberg);
[ISO/TS 16281](#ref-iso16281); [Fujiwara & Kawase 2006](#ref-fujiwara);
[NASA TM 2016](#ref-nasa-crown)).

Logarithmic crown drop (ISO/TS 16281 / Lundberg):

\[
z(x)
\;=\;
\frac{Q}{\pi L E'}\,
\ln\!\frac{1}{1-(2x/L)^2}
\]

\(Q\) load, \(L\) effective length, \(E'\) reduced modulus. The log
blows up at the ends — real rollers chamfer there. FDM cannot hold
that curve at exam length 11.2 mm.

**Printable proxy (this kit):** circular barrel, **short mid land**,
**through-bore**. A long conforming land on two flats is still a
slider. Lundberg / ISO/TS 16281 want the generator to *leave* the
race at the ends; FDM cannot hold the log curve, so we shorten the
flat and drop the ends.

| | Exam (0.4) | Source (1.0) |
|--|----------:|-------------:|
| Mid Ø (pack height) | 8.0 | 12.0 (floors to 8 at exam) |
| Mid land | **2.46** (22% of L, floor 6 nozzles) | same fraction |
| End drop \(\delta\) | **0.40** | 0.4–0.6 |
| End Ø | **7.20** | — |
| Through-bore | **Ø2.4** (6 nozzles) | same |
| End chamfer | 0.4–0.6 × 45° (slicer / knife) | 1.0 |
| Print | Standing. PETG HF. 4+ perimeters. Sand 400→1000 on the OD. | same |

![Crown](figures/print-kit-bearing/fig-b4-crown.svg)

**Fig. B4.** Mid-length is the only contact. Ends clear the keepers.
That is the “very small taper” that still obeys pure rolling on flats.

A 2–3° generator droop over L11.2 is \(\delta \approx
(L/2)\tan 2^\circ \approx 0.20\,\mathrm{mm}\) — same band.

---

## 4. Opening and bottom — what the cylinder slot did (fixed)

The first printed kit cut this (`cut_top_load_slots` /
`place_radial_cylinder`). That is gone. The compilers now cut a
U-window through the fence only. Kept here so the diagnosis stays
readable:

```text
place_radial_cylinder(Ø = roller + 0.40 + 0.80, L = roller + 0.40)
  centered at z_mid
  solid_combine cut from stator
```

Replace with a **downward U-window**:

| Feature | Target (exam) | Why |
|---------|---------------|-----|
| Race face | Flat, uncut | Bottom land |
| Race chamfer | 0.8 mm, 45°, ID and OD of the ring | No square corner under the roller |
| Window through fence | Stadium / rounded-rect, cut **from above**, stop at race \(Z\) | Opening ≠ trough |
| Funnel | \(20^\circ\)–\(30^\circ\) on the fence mouth | Drop-in. FilamentFeed / Bayer lead-in for snaps is the same angle family |
| Circumferential clearance | ≥ 0.80 mm/side (2 nozzles) after print | Spin. FDM slots close ~0.2–0.4 |
| Axial (end) clearance | 0.40 running, keepers stay | Walk, do not escape |
| Fence height | Keep **&lt; pack − 1.2** | Plate cannot sit on the fence if rollers shrink |
| Plate float | 0.20 modeled | Unchanged |

Print the rollers **standing** (circular layers on the OD). A lying
roller is layers on the race — that is a file. Vase / thick walls
([Bombermine](#ref-bombermine) on tapered printed rollers) is the
right slicer note.

---

## 5. Noise and friction (PLA is the limit, geometry is the first fix)

| Source | Number | What we do |
|--------|--------|------------|
| Dry FDM PLA CoF | ~0.15–0.25 typical, up to ~0.4 | Geometry first. Then dry PTFE on **races only**. |
| Oil / grease on FDM | CoF down to ~0.05 ([Hanon 2023](#ref-hanon-2023)) | Optional service. Do not oil a glow-PLA rotor skin. |
| Layer thickness | 0.20 mm is a CoF peak in that study | 0.12–0.16 mm on rollers if we reprint. |
| Orientation | Dominant for wear ([Srinivasulu 2024](#ref-srinivasulu-2024)) | Standing rollers. Race as XY (already). |
| Stick-slip | High \(\mu_s/\mu_k\) on rough PLA | Crown + flats + PTFE. Do not vapor-smooth a running fit. |
| Cage rub | Dry TRB torque rises when pocket clearance is tight ([Deng et al. 2018](#ref-deng-2018)) | Our wrap is worse than a tight cage. Open the sides. |

igus / iglidur filaments are a later **material** upgrade, not this
exam. Architecture stays when the plastic changes. This kit’s pair
is **PETG roller on PLA race** (§8) — not PLA-on-PLA and not
PETG-on-PETG.

---

## 6. Clip (unlocked — it is part of integration)

The shipped retainer is a printed-flat **C-washer**: D-hole 10.68,
journal 12.0, C-gap 3.2 (not side-entry). Axial assembly strain if
it were a closed hoop:

\[
\varepsilon_\mathrm{hoop}
\;=\;
\frac{D_\mathrm{shaft}-D_\mathrm{hole}}{D_\mathrm{hole}}
\;=\;
\frac{1.32}{10.68}
\;\approx\;
12.4\%
\]

PLA allowable snap strain is **2–3%**
([FilamentFeed June 2026](#ref-filamentfeed); Bayer / BASF annular
model). The gap turns this into bending at the back of the C, but
the rim is still ~4.9 mm thick for 0.66 mm of opening — a stiff
washer with a slot. That is why it feels awful.

Cantilever strain (the family we want):

\[
\varepsilon
\;=\;
\frac{1.5\,h\,Y}{L^2}
\qquad
\frac{L}{h}\ge 8\text{–}10
\qquad
r_\mathrm{root}\ge 0.5\,h
\]

Worked PLA target: \(h=1.6\), \(Y=0.8\), \(\varepsilon_\mathrm{allow}=0.025\)
\(\Rightarrow L=\sqrt{1.5\cdot 1.6\cdot 0.8/0.025}\approx 8.8\,\mathrm{mm}\).
Add finger tabs. Print flat. Assemble **radially** into the existing
groove. Journal stays a constant pass so the rotor still drops on.

**Do not:** metal circlip, closed hoop, side-entry narrower than the
journal, or a clip that rubs the rotor. Groove lip **45°** (hand
release + printable overhang). Journal tip **30°** lead-in.

Full earlier study: [PRINT_KIT_GDT.md](PRINT_KIT_GDT.md) § Clip.
That page said “do not change clip CAD this pass.” **This pass
changes it.**

---

## 7. Integration order (this branch — implemented)

Blades / helix / 0024-4.5/3.5 / 8-roller count / PCD under the roots
**stay**. Changed: pack interface and clip.

1. Stopped cutting the slot as a radial cylinder.
2. Flat race. U-window + funnel through the fence only. Witness Ø2.4
   through the outer keeper so the roller revolute still has a stator
   cylinder after the pocket is gone.
3. Barrel-crowned rollers (`solid_revolve` of a closed ring). Short
   mid land (~22% of L), 0.40 mm crown, Ø2.4 through-bore. Mid Ø
   still sets pack height. Bore radius stays **> 2.5 mm** away from
   the mid-land radius so the revolute matcher cannot pick the hole.
4. Replaced the C-washer with a printed E-clip / finger-tab circlip
   in the same groove.
5. **PETG HF Black** on rollers + clip. Stator stays PLA Orange
   (both race flats). Rotor / blades stay **PLA Glow** — do not PETG
   the spinning mass. Dry PETG. Hardened nozzle.
6. Reprint rollers standing; sand ODs; dry PTFE on the two flats
   is still optional (print-side, not CAD).
7. Then, and only if the crowned flats still skate under overturning,
   a matched-apex TRB (tapered race + tapered plate + tapered
   rollers). Not before.

Validate without reloading Cursor MCP:

```
node scripts/nbcad-cli.mjs exam --stage=stator
node scripts/nbcad-cli.mjs exam --stage=rollers
node scripts/nbcad-cli.mjs exam --stage=clip
node scripts/nbcad-cli.mjs exam --stage=kit
```

`solid_check` is the CAD error detector (timeline + welded manifold).

---

## 8. Materials — PETG on the pack, PLA Glow on the blades

Jeff allowed PETG on one or more surfaces and asked to **keep the
blades light** because a generator comes later. Density almost does
not decide this (catalog PLA 1.24, glow 1.26, PETG HF 1.27–1.28
g/cm³). **Where** the mass sits, and **which pair** rubs, does.

### 8.1 Polar inertia, not filament density

H-Darrieus blades sit at constant \(R\). The pack is under those
roots, so roller centers are at \(r \approx R\) too. Polar inertia:

\[
I_z \;\approx\; m_\mathrm{blades}\,R^2 \;+\; m_\mathrm{rollers}\,(PCD/2)^2 \;+\; I_\mathrm{plate}
\]

Exam scale 0.4, CAD-estimate solids: blades \(\sim 32\,\mathrm{cm}^3\),
eight hollow rollers \(\sim 4.6\,\mathrm{cm}^3\), plate \(\sim 15\,\mathrm{cm}^3\).
Blades still dominate \(I_z\) by about **7×** the roller pack at the
same radius. A later generator adds shaft inertia (small \(r\)) and
**load torque**. Cut-in is \(T = I\alpha\) — extra grams on the
blades hurt \(\alpha\) more than extra grams on a clip.

Hollow rollers still help: bore \(\varnothing 2.4\) on mid \(\varnothing 8\)
drops roller area by \(1-(1.2/4)^2 = 9\%\). PETG is \(\sim 2\%\)
denser than PLA Basic, so the hollow PETG pack is **~7% lighter**
than the old solid PLA cylinders. Direction is right. Do not chase
that 7% by PETG-ing the rotor.

### 8.2 Pairing table

| Part | Filament | Why |
|------|----------|-----|
| Stator (both race flats) | PLA Basic Orange | Dissimilar to PETG rollers on **both** races. Stator does not spin. |
| Rotor / blades / plate | PLA Glow Green | Upper race is the plate underside. Keep the spinning mass light. Glow is the rotor visual. |
| Rollers | **PETG HF Black** | Wear + dissimilar pair. Hollow + short land. |
| E-clip | **PETG HF Black** | Snap strain 3–5% vs PLA 2–3% ([FilamentFeed](#ref-filamentfeed)). PETG HF XY break strain 8.6%. |

Rejected:

| Pairing | Why not |
|---------|---------|
| PETG stator + PETG rollers | Lower race is PETG-on-PETG. Korean SPE: same-material PLA was worse than PETG, but same-material PETG is still same-on-same. |
| PETG stator + PLA rollers | Lower race dissimilar; **upper** race is still glow-PLA on PLA rollers. |
| PETG washer under the plate | Extra spinning part. If rollers are PETG, the upper land becomes PETG-on-PETG. |
| PETG blades / PETG plate | Mass at \(R\). No bearing benefit. Generator inertia budget. |

Do not add a PETG race insert. The plate *is* the upper race.

### 8.3 Why PETG on the roller, not the race

[Korean SPE 2024](#ref-kspe-2024) (dry FDM reciprocating): PLA-on-PLA
had higher CoF and wear than PETG-on-PETG; PLA deposited from
frictional heat. A **PLA block + PETG pin** cut PLA wear versus
PLA-on-PLA (PETG pin wore more — that is the sacrificial roller).
[Hanon 2023](#ref-hanon-2023) dry FDM PLA CoF \(\sim 0.15\)–\(0.4\).
[Polymer Korea 2023](#ref-polymer-korea-2023) abrasive pin-on-disk:
PETG wore less than PLA across the load/speed matrix.

So: put PETG on the **small, replaceable** bodies (8 rollers + clip).
Leave both large flats PLA. Geometry (§2–3) still comes first —
PETG in a conforming trough is still a journal.

Bambu PETG HF TDS ([Bambu PETG HF](#ref-bambu-petg-hf)): density
1.28 g/cm³ (catalog 1.27), HDT 69 °C / 0.45 MPa, UV + water, XY
elongation \(8.6\pm 1.2\%\). **Dry before print** (65 °C / 8 h oven,
or 75–85 °C / 12 h heatbed). Hardened nozzle (glow *and* PETG).
Do not anneal the crowned rollers — the TDS warns shaped PETG HF
warps. PETG HF is stiffer / less tough than PETG Basic (impact XY
31.5 vs 52.7 kJ/m²); it is the preset already in the catalog
(`bambu.petg.hf.black`).

### 8.4 Short land + hollow (CAD numbers)

`solid_revolve` of a closed ring, exam 0.4:

```text
bore Ø2.4  →  end Ø7.20  →  mid Ø8.00 × L2.46  →  end Ø7.20  →  bore
crown drop = nozzle.clamp(0.30, 0.40) = 0.40
land       = max(6×nozzle, 0.22×L) = 2.46
|r_mid − r_bore| = 4.0 − 1.2 = 2.8 > 2.5   // revolute matcher
```

Print standing (circular layers on the OD). The bore is a mass hole,
not a shaft. Witness holes through the outer keeper stay Ø2.4 for
the stator-side revolute pick.

---

## 9. References

### Rolling-element geometry

<a id="ref-timken"></a>
The Timken Company. *Tapered Roller Bearing Catalog.*
Apex coincidence: race and roller generators meet on the axis.
<https://www.timken.com/wp-content/uploads/2016/10/Timken-Tapered-Roller-Bearing-Catalog.pdf>

<a id="ref-iso355"></a>
ISO 355:2007. *Rolling bearings — Tapered roller bearings —
Boundary dimensions and series designations.*

<a id="ref-harris"></a>
Harris, T. A. & Kotzalas, M. N. (2006).
*Rolling Bearing Analysis.* 5th ed. CRC Press. Ch. 1–5 (kinematics,
Hertz, roller guidance).

<a id="ref-lundberg"></a>
Lundberg, G. (1939).
Elastische Berührung zweier Halbräume.
*Forschung auf dem Gebiet des Ingenieurwesens* **10** 201–211.

<a id="ref-iso16281"></a>
ISO/TS 16281:2008.
*Rolling bearings — Methods for calculating the modified reference
rating life for universally loaded bearings.* Logarithmic roller
profile.

<a id="ref-fujiwara"></a>
Fujiwara, H. & Kawase, T. (2006).
Logarithmic profile of rollers in roller bearing and optimization
of the profile.
*Trans. JSME Ser. C* **72** 3022–3029.
doi:[10.1299/kikaic.72.3022](https://doi.org/10.1299/kikaic.72.3022)

<a id="ref-nasa-crown"></a>
Poplawski / NASA (2016).
*Effect of Roller Geometry on Roller Bearing Load-Life Relation.*
<https://ntrs.nasa.gov/api/citations/20160000341/downloads/20160000341.pdf>

<a id="ref-deng-2018"></a>
Deng, S., et al. (2018).
Modeling the frictional torque of a dry-lubricated tapered roller
bearing considering the roller skewing.
*Friction* **7** 260–272.
doi:[10.1007/s40544-018-0232-8](https://doi.org/10.1007/s40544-018-0232-8)

### 3D-printed bearings and PLA tribology

<a id="ref-hanon-2023"></a>
Hanon, M. M., et al. (2023).
Tribological characterisation and modelling for FDM polymeric
structures under lubrication conditions.
*Polymers* (PMC10610781). Dry CoF ~0.15–0.2; oil ~0.05.
<https://pmc.ncbi.nlm.nih.gov/articles/PMC10610781/>

<a id="ref-srinivasulu-2024"></a>
Srinivasulu, N., Suresh, G. & Rao, K. V. (2024).
Optimizing tribological performance of 3D-printed PLA through
process parameter analysis.
*Iranian Polymer Journal*.
doi:[10.1007/s13726-024-01412-8](https://doi.org/10.1007/s13726-024-01412-8)

<a id="ref-bombermine"></a>
Bombermine (2025).
Printable tapered-roller *thrust* bearing (races are cones; vase
rollers). Printables model 1520277.
<https://www.printables.com/model/1520277-thrust-bearing>

<a id="ref-igus"></a>
igus. iglidur 3D-print filaments — lower CoF than commodity PLA.
Later material table, not this exam.

### Snaps / clip

<a id="ref-filamentfeed"></a>
FilamentFeed (June 2026).
*Snap Fit Design for 3D Printing.*
PLA strain 2–3%; \(\varepsilon=1.5 h Y / L^2\); annular &lt; Ø30 print
flat; lead-in 30°, retention 45°.
<https://filamentfeed.com/article/snap-fit-design-3d-printing-guide-june-2026>

<a id="ref-bayer"></a>
Bayer / BASF. *Snap-Fit Design Manual.*
Annular \(\varepsilon\approx\Delta D/D\).

<a id="ref-sovol"></a>
Sovol / HMaking cantilever notes (2025–26).
\(L/t \ge 8\)–\(10\) PLA; fillet \(r\ge 0.5 t\); flex in XY.

### PETG / PLA pairing and filament

<a id="ref-kspe-2024"></a>
Kim et al. (2024).
A study on the wear phenomena of PLA and PETG materials for 3D
printing in non-lubricated condition.
*J. Korean Soc. Precis. Eng.* **41** 145–151.
doi:[10.7736/jkspe.023.119](https://doi.org/10.7736/jkspe.023.119)
PLA-on-PLA worse CoF/wear than PETG; PLA block + PETG pin reduced
PLA wear.

<a id="ref-polymer-korea-2023"></a>
Investigation on wear behavior of PETG and PLA parts fabricated
with fused deposition modelling under different wear conditions.
*Polymer Korea* **47** 257–263 (2023).
doi:[10.7317/pk.2023.47.3.257](https://doi.org/10.7317/pk.2023.47.3.257)
PETG lower abrasive wear than PLA across the tested loads/speeds.

<a id="ref-bambu-petg-hf"></a>
Bambu Lab. *PETG HF Technical Data Sheet.*
Density 1.28 g/cm³; HDT 69 °C (0.45 MPa); XY elongation
\(8.6\pm 1.2\%\); dry 65 °C / 8 h before print.
<https://3d.nice-cdn.com/upload/file/Bambu_PETG_HF_Technical_Data_Sheet.pdf>
