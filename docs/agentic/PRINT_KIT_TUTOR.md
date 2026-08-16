# Print-kit tutor (CAD synthesis exam)

**Benchmark #1** in the agentic suite
([INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)). This is the reusable
integration test for **AI → fully printable mechanical CAD**. It is the
curriculum, the worked example, and the grader.

The first kit (scattered shaft / 608 coupon / lone C-loft) failed that bar:
it did not assemble, the blade was not a rotor, and nothing would spin
without a hidden metal bearing. This exam builds an **even spinner** instead.

Assembly gap (mates, instances, configs): [ASSEMBLY.md](ASSEMBLY.md).

## What to build

A six-body **fully printed even spinner** (spec
`scripts/fixtures/print-kit-tutor.spec.json`), placed **assembled on one
axis**:

| Body | Role | How it mates |
|------|------|----------------|
| Base | Ø64 plate, 45° conical thrust cup, 3 posts at 120° | Cup takes the shaft cone; posts take the top plate |
| Shaft | Matching cone (tip lifted 0.3 mm), Ø8 journal, Ø16 shoulder | Spins in the cup and the printed bushing |
| Rotor | Hub Ø8.4 on the shoulder + two helical buckets at 180° | Slides onto the shaft; even so it can run true |
| Top plate | Ø64 deck, Ø6.4 post holes, Ø14.4 bushing seat | Drops onto the three posts |
| Printed bushing | Ø8.4 / Ø14 × 4 sleeve | Radial bearing in the plate; no metal 608 required |
| Cap | Ø20 washer | Slips onto the journal and keeps the stack down |

Every printed-to-printed running or slip fit is **+0.40 mm diametral** (one
0.4 mm Bambu nozzle). Do not use press fits.

Assembly order: **base → shaft → rotor → top plate → bushing → cap**.

Print each body in its own orientation (base/plate flat, shaft on the cone,
bushing as a ring, rotor on the hub face). The exam shows the assembled
stack so the mechanism is readable. noBS CAD cannot store a second print
layout.

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
3. Build the assembled stack
4. Print a thrust bearing contour
5. Keep the machine even
6. Print the bearings too
7. The rotor is a part that mounts
8. Export a printable package

Later (not this exam): catalog metal bearings from a standard table at
larger sizes.
