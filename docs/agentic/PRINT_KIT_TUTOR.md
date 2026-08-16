# Print-kit tutor (CAD synthesis exam)

**Benchmark #1** in the agentic suite
([INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)). This is the reusable
integration test for **AI → printable mechanical CAD**. It is the
curriculum, the worked example, and the grader.

## What to build

A four-body **FDM print-tolerant journal kit** (spec
`scripts/fixtures/print-kit-tutor.spec.json`):

| Body | Print orientation | Why |
|------|-------------------|-----|
| Ø8 journal shaft, Ø16 flange, 45° tip | Stand on the flange | No square barb, no horizontal holes |
| 608 bushing Ø8.4 / Ø22 / 7 | Stand as a ring | Bore is an XY circle |
| Housing with Ø22.4 × 7.4 seat + Ø10 through | Flat, pocket from the top | Slip + shoulder; metal 608 optional |
| Helical C loft, 90° twist | Stand on the C end | Not a 2D vane / single extrusion |

Every printed-to-printed running or slip fit is **+0.40 mm diametral** (one
0.4 mm Bambu nozzle). Do not use press fits. Leave slicer XY hole
compensation at 0 unless a gauge print says otherwise.

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
3. Orient for the bed, not the assembly
4. Functional holes are XY circles
5. Print real machine elements (journal + 608 seat)
6. Do not stop at a 2D extrusion
7. Export a 3MF manufacturing package

Scale-up uses the same rules: helical Savonius, double-D drive, thrust
washers, C-clip, integrated bosses.
