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
| Straight NACA in a Ø90 cage | Section was right; girth was wrong. Posts Ø8 vs chord 12. Prismatic blades idle most of the rev. |
| Helical NACA 0021 on a Ø72 stand | 60° helix, chord 16 mm, span 48 mm, posts Ø5 on R32. |

## 2. Design process

- **Architecture:** Helical H-Darrieus, directionless (no yaw). Symmetric section because α reverses each rev. Straight extrusion is idle for most of the rev at this Re.
- **Airfoil:** NACA 0021 (t/c 0.21). 2026 band is t/c 21–24%. TE blunt to 0.8 mm.
- **Rotor:** N=3, c=16 mm, R=24 mm, span=48 mm, helix 60°, σ=0.318. Frame/rotor **1.42**, post/chord **0.31**.
- **Fits:** +0.40 mm every printed running/slip interface. Printed cup+land + printed sleeve. No 608.
- **Service finish:** layer lines spanwise; sand PLA 400→1000 on skins.

## 3. Final product

Nine coaxial bodies: base → shaft → hub → wing ×3 → top plate → bushing → cap.

Wing bbox (exam): 33.2 × 24.8 × 48.0 mm; faces=24.

Open in CAD: `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad`

## 4. Printing cost (plastic / material)

Assumptions: Bambu PLA Basic, 1.24 g/cm³, $20/kg, print-volume factor 0.42.

| | Value |
|--|------:|
| CAD solid (estimate) | 63.3 cm³ |
| Estimated print mass | 33.0 g |
| Filament cost | **$0.66** |

Electricity and machine time are not priced. No additional hardware.
