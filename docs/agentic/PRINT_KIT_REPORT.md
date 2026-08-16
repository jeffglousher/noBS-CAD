# Print Kit Tutor — design report

Spec `fdm-print-vawt` · Printed VAWT · nozzle 0.4 mm · clearance +0.4 mm

Worked example from `npm run test:mcp-print-kit`. Live copy:
`%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor-design.md`.
Contract: [PRINT_KIT_DESIGN.md](PRINT_KIT_DESIGN.md).

## 1. Iteration log

| Iteration | Why it failed / what changed |
|-----------|------------------------------|
| Print-bed scatter | Parts did not assemble on one axis. Not a machine. |
| Colliding spinner | Rotor swept the posts. Could not rotate. |
| Helical C-buckets | Leftover loft, not a mount, not an airfoil. |
| Hoop sector r20–r28 | Concentric C. Concave faces the axis. No net torque. |
| Turntable / lazy Susan | Competent bearings, no wing. The frame had nothing to do. |
| Flat plate 12×2.4×32 | A vane, not a 2026 symmetric section. Directionless VAWT needs an airfoil. |
| NACA 0021 on the two-bearing stand | Thick symmetric section (t/c 0.21), blunt TE 0.8 mm, tenon in hub socket. |

## 2. Design process

- **Architecture:** H-Darrieus, directionless (no yaw). Symmetric section because α reverses each rev. Savonius-only is a drag toy. A HAWT section is the wrong physics.
- **Airfoil:** NACA 0021 (t/c 0.21). 2026 VAWT dynamic-stall work favors t/c 21–24%, xt/c 27.5–35%, reduced LE radius (NACA 0024–4.5/3.5 best in that study). NACA 0021 is the printable stand-in; TE blunt to 0.8 mm (≥ 2 nozzles).
- **Rotor:** N=3, c=12 mm, R=24 mm, span=32 mm, σ=0.239. Desk Re ~5e3–1e4 at 3–5 m/s, TSR ~2. Tip must clear post inner wall.
- **Fits:** every printed-to-printed running/slip interface is +0.4 mm diametral, including the tenon. No press. No metal 608.
- **Bushings:** printed 45° cup + land (thrust), printed sleeve Ø8.4/14 (upper radial). Prefer bushings over rollers at this size. PLA-on-PLA is a demo spin.
- **Service finish:** blades printed so layer lines run spanwise; sand PLA 400→1000 on skins (or ABS vapor, not immersion). Do not vapor-smooth a running fit and keep +0.4.
- **Later, not this exam:** catalog bearings, higher AR, modified 0024–4.5/3.5, optional adaptive Darrieus–Savonius starter (*Flow* 2026).

## 3. Final product

Nine coaxial bodies, assembly order: base → shaft → hub → wing → wing → wing → top_plate → bushing → cap.

| Body | Count | Role |
|------|------:|------|
| Base + posts | 1 | Two-bearing stand. Cup + 3× Ø8 on R38. |
| Shaft | 1 | Cone/land thrust, journal, shoulder, double-D. |
| Hub | 1 | Sits on the shoulder. Three sockets. |
| Wing (NACA 0021) | 3 | Mid-chord R24, chord tangential, tenon drop-in. |
| Top plate | 1 | Posts through, windows, bushing land. |
| Bushing | 1 | Printed sleeve. |
| Cap | 1 | 0.20 float retain. |

Wing bbox (exam): 16.4 × 18.8 × 32.0 mm; faces=30. Bodies=9.

## 4. Printing cost (plastic / material)

Assumptions: Bambu PLA Basic, 1.24 g/cm³, $20/kg, print-volume factor 0.42 (3 walls + ~15% gyroid on bulky parts; blades nearer solid — factor is a kit average).

| | Value |
|--|------:|
| CAD solid (estimate) | 94.6 cm³ |
| Estimated print mass | 49.3 g |
| Filament cost | **$0.99** |

3MF: `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.3mf`  
Project: `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad`

Electricity and machine time are not priced. No additional hardware.
