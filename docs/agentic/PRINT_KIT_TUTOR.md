# Print-kit tutor (CAD synthesis exam)

**Benchmark #1** in the agentic suite
([INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)). This is the reusable
integration test for **AI → fully printable mechanical CAD**. It is the
curriculum, the worked example, and the grader.

The first kits failed that bar: a print-bed scatter, an assembled spinner
that collided, leftover helical C-buckets, a concentric hoop, a turntable
that threw the frame away, then a flat plate sold as a wing. The frame
was the good part. The wing has to be a **named airfoil** that uses it.
Contract: [PRINT_KIT_DESIGN.md](PRINT_KIT_DESIGN.md). Fits:
[PRINT_KIT_GDT.md](PRINT_KIT_GDT.md). Agents start with
`prompts/get model_print_kit` — that recipe is adversarial.

Assembly gap (mates, instances, configs): [ASSEMBLY.md](ASSEMBLY.md).

## What to build

A nine-body **printed VAWT** (spec
`scripts/fixtures/print-kit-tutor.spec.json`, id `fdm-print-vawt`),
placed **assembled on one axis**:

| Body | Role | How it mates |
|------|------|----------------|
| Base | Ø90 × 6 plate, 45° cup (r5), 3× Ø8 posts on R38 | Cup centers the shaft. Posts locate through the top plate and stand 2 mm proud. |
| Shaft | Male cone r4.8, Ø13 × 0.8 land (0.20 float), Ø8 journal, Ø16 shoulder, double-D 6.0 in the hub zone | Land takes thrust; journal runs in the sleeve; double-D drives the hub |
| Hub | Ø28 × 8, Ø8.4 bore, **sits on the shoulder**, double-D 6.4, 3 sockets at 60°/180°/300° | The wing mount. Sockets open to the OD. |
| Wing ×3 | **NACA 0021** (t/c 0.21), chord 12 mm tangential, blunt TE 0.8 mm, span 32, mid-chord R24, 7.6 × 4.8 tenon | Drops into a hub socket. Stands in a bay between the posts. |
| Top plate | Ø90 × 6 at z=52, Ø8.4 post holes, Ø16 windows over the blades, Ø14.4 × 4 seat | Drops onto the three posts; windows keep the rotor visible; seat holds the sleeve |
| Printed bushing | Ø8.4 / Ø14 × 4 sleeve | Sits on the land; upper radial bearing; no metal 608 |
| Cap | Ø20 × 2.4 washer, 0.20 float above the plate | Slips onto the journal and keeps the stack down |

Every printed-to-printed running or slip fit is **+0.40 mm diametral** (one
0.4 mm Bambu nozzle). Do not use press fits. Running faces have modeled
gaps; do not occupy the same volume.

Assembly order: **base → shaft → hub → three wings → top plate → bushing → cap**.

Print each body in its own orientation (base/plate flat, shaft on the
land, bushing as a ring, hub on its face, wings standing so layer lines
run spanwise). The exam shows the assembled stack so the mechanism is
readable. noBS CAD cannot store a second print layout. The exam also
writes a design report (iteration + plastic cost) next to the 3MF.

## How to rerun

Headless (CI and local goldens — no UI attach):

```powershell
$env:OCCT_ROOT = "$PWD\vcpkg_installed\x64-windows"
$env:Path = "$env:OCCT_ROOT\bin;$env:Path"
cargo test --manifest-path mcp-server/Cargo.toml print_kit_tutor
npm run test:mcp-print-kit
```

Optional live desktop: `node scripts/mcp-print-kit-tutor.mjs --live`

The exam also writes a reusable project next to the 3MF (override with
`NBCAD_PROJECT_OUT`):

```text
%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor.nbcad
```

Agents start with `prompts/get model_print_kit`.

## Lessons the grader checks

1. Clearance is a design input
2. No FDM press fits
3. The wing uses the frame (sockets, posts through plate, sweep clears posts)
4. Print a thrust bearing that can spin (smaller male cone + land, not a lifted same-angle cone)
5. Keep the machine even (3 posts + 3 wings in the bays, double-D drive)
6. Prefer printed bushings (no hidden 608)
7. The wing is a part that mounts
8. The section is a 2026-appropriate airfoil (not a plate or hoop)
9. Publish the design report (iteration + plastic cost)
10. Export a printable package

Later (not this exam): catalog metal bearings from a standard table at
larger sizes.
