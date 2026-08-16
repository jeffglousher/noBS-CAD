# Print-kit tutor (CAD synthesis exam)

**Benchmark #1** in the agentic suite
([INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)). This is the reusable
integration test for **AI → fully printable mechanical CAD**. It is the
curriculum, the worked example, and the grader.

The first kits failed that bar: a print-bed scatter, then an assembled
spinner that collided, then a **competent frame around a leftover VAWT**.
A Ø90 post-and-plate cage with two helical C-buckets is a grader checkbox,
not a product. Corrections are in [PRINT_KIT_GDT.md](PRINT_KIT_GDT.md).

Assembly gap (mates, instances, configs): [ASSEMBLY.md](ASSEMBLY.md).

## What to build

A six-body **printed turntable** (lazy-Susan / paint stand, spec
`scripts/fixtures/print-kit-tutor.spec.json`, id `fdm-print-turntable`),
placed **assembled on one axis**:

| Body | Role | How it mates |
|------|------|----------------|
| Base | Ø48 × 6 foot, 45° cup (r5), Ø3 relief | Cup centers the shaft. No posts. |
| Shaft | Male cone r4.8, Ø13 × 0.8 land (0.20 float), Ø8 journal, Ø16 shoulder, double-D 6.0 in the platter zone | Land takes thrust; journal runs in the sleeve; double-D drives the platter |
| Platter | Ø72 × 6, Ø8.4 bore, **sits on the shoulder**, double-D 6.4, rim well Ø64 × 1.2, 3× Ø16 wells on R22 at 120° | The part. Larger than the foot. Wells keep mass even. |
| Keeper | Ø28 × 6 collar at z=23.5, Ø8.4 journal, Ø14.4 × 4 seat (2 mm land) | Small collar — not a second lid. Seat holds the sleeve. |
| Printed bushing | Ø8.4 / Ø14 × 4 sleeve | Sits on the land; radial bearing; no metal 608 |
| Cap | Ø20 × 2.4 washer, 0.20 float above the keeper | Slips onto the journal and keeps the stack down |

Every printed-to-printed running or slip fit is **+0.40 mm diametral** (one
0.4 mm Bambu nozzle). Do not use press fits. Running faces have modeled
gaps; do not occupy the same volume.

Assembly order: **base → shaft → platter → keeper → bushing → cap**.

Print each body in its own orientation (base/platter/keeper flat, shaft on
the land, bushing as a ring). The exam shows the assembled stack so the
mechanism is readable. noBS CAD cannot store a second print layout.

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
3. Build a product, not a cage (platter larger than the foot; keeper is a collar)
4. Print a thrust bearing that can spin (smaller male cone + land, not a lifted same-angle cone)
5. Keep the machine even (3 wells at 120° and double-D drive)
6. Print the bearings too
7. The platter is a part that mounts
8. Export a printable package

Later (not this exam): catalog metal bearings from a standard table at
larger sizes.
