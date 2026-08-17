# Print-kit aero — why this section, this rotor, this wind

Research contract for the live kit. Numbers:
`scripts/fixtures/print-kit-tutor.spec.json`. Appearance (organic
root / tip landing) is not aero:
[PRINT_KIT_ORGANIC_REFERENCE.md](PRINT_KIT_ORGANIC_REFERENCE.md).

This is **not** “the ideal airfoil for windmills.” It is the best
**coupled symmetric section in the published low-TSR Darrieus design
space** for a printed, directionless, low/medium-speed machine. A HAWT
or a high-TSR Darrieus would pick a different foil.

## Operating point we designed for

| Quantity | Kit intent | Why |
|----------|------------|-----|
| Architecture | Helical H-Darrieus, 3 blades, axis Z | No yaw. Wind from any azimuth. |
| Tip-speed ratio λ | ~2.0–3.0 (low / medium) | Urban / rooftop / desk-scale Re. High λ needs a thinner foil. |
| Reynolds (exam 0.4) | chord ~30 mm, V ~3–8 m/s → Rec ~1e4–4e4 | Thick section + high solidity, or it will not start. |
| Reynolds (scale 1.0) | chord 74 mm | Still low-Re. Same section family. |
| Solidity σ | ~0.38 (Nc / πD) | Low-TSR torque and self-start. Cap 0.45 so it does not choke at medium λ. |
| Helix | 60° from a 30° root | Best mean Cp among 60/90/120 in 3D CFD; still covers the rev. |

A Darrieus blade sees **reversing α every revolution**. Camber that
helps the upwind pass hurts the downwind pass. That is why the section
is symmetric (NACA 00xx), not a HAWT camber.

## Source of the section match

The live section is **NACA 0024-4.5/3.5**:

| Symbol | Value | Meaning |
|--------|-------|---------|
| 0024 | t/c = 24% | Maximum thickness |
| 4.5 | LE index I | Sharper nose than the default I = 6 |
| 3.5 | xt/c = 35% | Max thickness at 35% chord (aft of the stock 30%) |

**Primary sources (same group, same 126-foil family):**

1. Tirandaz & Rezaeiha, *Renewable Energy* **173** (2021) 422–441.
   [doi:10.1016/j.renene.2021.03.142](https://doi.org/10.1016/j.renene.2021.03.142)
   — 252 URANS cases, 126 symmetric modified-NACA foils, validated on
   three experiments. At **λ = 2.5** (dynamic stall) the optimum moves
   from NACA 0018-4.5/2.75 to **NACA 0024-4.5/3.5**: thicker, max
   thickness aft, I stays 4.5. Dropping I from 6.0 to 4.5 raises Cp.
2. Tirandaz, Rezaeiha & Micallef, *Wind Energy Science* **8** (2023)
   1403–1424. [doi:10.5194/wes-8-1403-2023](https://doi.org/10.5194/wes-8-1403-2023)
   — 630 transient runs. As λ **rises**, the optimum **thins** (24% →
   10%) and xt moves forward. I stays 4.5. Our kit is the **low-λ
   end** of that map, not the high-λ end.
3. Tirandaz, Rezaeiha & Micallef, *Energies* **19**(7) 1615 (2026).
   [doi:10.3390/en19071615](https://doi.org/10.3390/en19071615)
   — same coupled parameters. Thick + aft xt + lower I shifts stall
   from an abrupt leading-edge burst to a slower trailing-edge process,
   weakens the dynamic-stall vortex, and cuts unsteady load. Best case
   in that space: **NACA 0024-4.5/3.5**, **+73% Cp** vs NACA 0018-6.0/3.0.

We build that foil with the Ladson / NASA TM 4741 modified 4-digit
ordinates (I sets the nose radius, xt splits the front/aft polynomials),
then blunt the last quarter-chord to **≥ 2 nozzles** so FDM can print
it and the solid stays a manifold.

**What we did not pick, and why:**

| Alternative | Why not for this kit |
|-------------|----------------------|
| Stock NACA 0021 (I=6, xt=30%) | Default helicopter foil. Loses the coupled I/xt gain at low λ. |
| NACA 0018-4.5/2.75 | Optimum at λ ≈ 3.0, not 2.5. Thinner = worse start at this Re. |
| NACA 4412 / cambered HAWT | One-way camber. Directionless VAWT sees −α every half-rev. |
| Gurney / hybrid flap on 0021 | *Results in Engineering* (2026) can raise low-Re Cp. A flap is extra print, extra fatigue, not a substitute for the section. |
| Whale tubercles | Mixed 2023–2025 VAWT results; 0.4 mm nozzle cannot hold 0.06c amplitude at exam chord. |
| Savonius-only | Directionless drag machine. Low Cp. Optional starter, not the wing (Gu / MacDonald / Tang, *Flow* 6, 2026). |

## Other geometry decisions (capability)

| Decision | Research / reason |
|----------|-------------------|
| 3 blades, 120° | Odd count, even torque, no preferred azimuth. |
| Chord 74/54 (exam 29.6/21.6), σ ≈ 0.38 | At Rec ≲ 1e5, longer chord / higher solidity raises low-λ torque (e.g. *Sustainability* 14:2623; high-σ start vs low-σ peak-Cp trade). 0.38 sits in the 0.25–0.45 urban band. |
| Drafted chord (root > tip) | Standing print. Also a mild taper the way a real blade is built. |
| 60° helix from 30° root | Peng et al., *Energies* **14** 393 (2021): among 60/90/120°, **60° had the best mean Cp**; larger helix is smoother but peaks later. Mid-helix stays at 60° azimuth so the machine stays even. |
| Mid-chord on the cylinder, chord tangent | H-Darrieus definition. A hoop-sector “scoop” faces the axis and makes no torque. |
| Open tips, short taper to a flat landing | Print sit + less tip vortex than a square cut. Not an endplate (those are a later study). |

## Durability (loads, not just Cp)

Capability without a bearing is a spinner that dies.

| Load | What takes it | Why this way |
|------|---------------|--------------|
| Torque about Z | One-piece rotor + 8 radial rollers | Helix cuts torque ripple (Battisti / Marsh / Peng). Less cyclic load on PLA. |
| Thrust / weight | Flat pack, pack height = roller Ø | Not a tall drum. Not a cone. |
| Overturning from the tips | Same pack, PCD **under the blade roots** | Couple across the disk. An inboard pack leaves the plate as a cracker. |
| Reversing α / dynamic stall | Thick aft 0024-4.5/3.5 | The 2026 *Energies* result is also a **load** result: weaker DSV, less unsteady bending. |
| Root print cliff | Organic 1.28× blend (appearance + fillet) | Stress riser, not aero. |
| Radial escape of rollers | Inner/outer keeper walls | Survive the top-load cut. |
| FDM material | PLA demo. Sand skins 400→1000 | As-printed Ra 10–25 µm is Cd. Vapor-smooth would eat the modeled running fit. |

PLA-on-PLA is a demo spin. A later metal 608 table is in
[ASSEMBLY.md](ASSEMBLY.md). The **architecture** (wide flat pack under
the roots, helix, thick section) is what you keep when the material
upgrades.

## Wind from every direction — including oblique

**Horizontal azimuth (any compass heading): already designed for this.**
A VAWT does not yaw. Three identical symmetric helical blades at 120°
produce the same family of torque from any incoming heading. A HAWT
section or a one-sided vane would break that.

**Oblique / skewed wind (not horizontal — rooftop, canyon, tilt):**
already usable, and sometimes **better** than pure crossflow.

- Mertens (and follow-on): an H-VAWT in skewed flow can **gain** Cp
  because the downwind blade is less buried in the upwind wake.
- Orlandi, Collu, Zanforlin, Shires, *J. Wind Eng. Ind. Aerodyn.*
  (2015), 3D URANS: the power gain in skew is on the **downwind**
  pass; the lower part of the blade sees cleaner flow.
  [Strathprints copy](https://strathprints.strath.ac.uk/71845/1/Orlandi_etal_JWEIA_2015_3D_URANS_analysis_of_a_vertical_axis_wind_turbine_in_skewed_flows.pdf)

The **60° helix** is the modification that makes shifting and skewed
urban wind into shaft motion instead of a once-per-rev slam: at any
instant some span station is at a working α. Straight H-rotors have
dead azimuths and higher fatigue. Helical urban-wake prototypes
(2025–26) report the same: smoother torque, better start, not a
higher peak Cp.

**What we would not add for “more directions”:**

| Change | Why not now |
|--------|-------------|
| Yaw bearing / tail | HAWT habit. This machine must not care. |
| Camber or a one-way flap | Breaks reverse-α. |
| 90–120° helix | Smoother, but Peng 2021: worse mean Cp than 60° and the peak moves to higher λ. |
| Inner Savonius / hybrid | Real low-λ starter (2025–26 hybrid papers). Extra body, extra drag at medium λ, extra print. Optional later, not the wing. |
| Sphere / Φ-rotor | Better some 3D inflow. Not a flat-print + standing-plate kit. |

So: **right now it takes wind from any heading and from moderate
oblique tilt**, and it turns that into rotation about Z through the
printed thrust pack. The next capability step is not a new foil. It is
a hybrid starter or a material upgrade on the same pack.
