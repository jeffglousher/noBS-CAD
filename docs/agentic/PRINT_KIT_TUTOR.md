# Print-kit tutor (CAD synthesis exam)

**Pipeline #1** in the MCP goldens
([INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)). This is a repeatable
command sequence that builds a printable mechanical assembly. Same spec,
same tool order, same kit. It is useful context for an agent. It is not
a test of AI capability.

Contract: [PRINT_KIT_DESIGN.md](PRINT_KIT_DESIGN.md). Fits:
[PRINT_KIT_GDT.md](PRINT_KIT_GDT.md). The recipe is
`prompts/get model_print_kit`.

The exam now **does** form a CAD assembly (components, occurrences, a
revolute). See [ASSEMBLY.md](ASSEMBLY.md). It starts with
`cad_new_project` and fails if the scene is not empty — do not continue
a recovered or older Print Kit Tutor. Construction planes and finished
loft sketches are hidden before the `.nbcad` is written so File → Open
shows the merged-stator kit, not orange datum stacks.

## What to build

A **printed VAWT assembly** (spec
`scripts/fixtures/print-kit-tutor.spec.json`, id `fdm-print-vawt`).
Linear numbers are the Bambu Lab X2D-max design (256×256×260, 8 mm
margin). `spec.scale` shrinks the source (exam default **0.4**). Feature
floors (roller Ø8, roller length 8, TE 0.8, 4-nozzle walls, plate 3.2 mm, base 3.2 mm) are clamped. Pack height is the roller diameter.

| Part | Role | How it mates |
|------|------|----------------|
| Stator | Thin Y-frame + race ring with keeper walls + open top-load fence + constant journal + snap groove, one piece | Grounded. Print flat. |
| Rotor | Thin root plate out to the blades (underside = upper thrust race), organic airfoil roots (appearance), 3 helical **NACA 0024-4.5/3.5** ending on that plate, **one body**, open tips with a short taper to a flat landing | Plate bore = journal + running. Print standing on the plate. PLA Glow. |
| Rollers | 6 **radial-axis** cylinders, min Ø8, pack height = Ø, large PCD **under the blade roots**, captured by keepers | Drop into top-load slots. Print standing. |
| Retainer | Clocked C-clip (D-hole + C-gap) | Snaps into the journal groove. Pull to remove. Does not rub the rotor. Clip CAD unchanged this pass — see the GDT clip study. |

Fits are **per role** (running / PIP / slip) and per whether
the parts share a plate. Assembled running +0.40. Top-load slots add two
nozzles. Same-plate PIP +0.80 — do not PIP a lying roller. Every
bed-printed locate gets a 0.80 mm elephant-foot lead-in. Do not
nest the plate around the rollers on the bed. Slicer XY hole compensation
stays 0. No metal 608s. No FDM press fits. No separate axle disk + cage
disk. No tall drum. No standing-Z pucks.

Assembly order: **stator → rollers into the top-load slots → drop rotor over the journal → snap C-clip**.

Then: one `assembly_create_component` per **moving** body (stator,
rotor, each roller, retainer). That call already inserts
the root occurrence — a second `assembly_create_occurrence` duplicates
every part. Ground the stator. **Revolute** `rotor_spin` (plate bore ↔
short journal) and each roller about its **radial**
axis — not a blade spar. **Rigid** `retainer_sit` on the journal
shoulder. Ship an A3 assembly drawing with notes.

Print each functional part in its own orientation on **one** plate.
Rollers print standing (axis Z). The exam **wipes**
`%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor\` first (retired
five-plate names plus `02-shaft` / `03-hub` / `04-wings` / `05-plate` /
`06-bushing` / `07-cap` and any assembled `Print-Kit-Tutor.3mf`), saves
the assembled `.nbcad`, lays the parts out, then writes `01-kit.3mf`
(PLA Orange + PLA Glow) and a design report next to the project.

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

Desktop crash recovery can reopen an older nest (tan base, red disc,
orange helix planes). That is not this kit. **File → New**, then
**File → Open** the path above. The current kit is an orange Y-frame /
axle / cage / retainer and a glow-green one-piece rotor on a thin thrust
pack, with 0.20 mm axial float at every running land. Switch workspace to
**Assembly** to see the five parts. Rollers drop onto the flange; the
rotor drops on the pack; cage height matches roller height.

Agents start with `prompts/get model_print_kit`.

## Lessons the grader checks

1. Start from a blank document (`cad_new_project`, 0 bodies; hide datums)
2. Fits are per role and material (running +0.40, slip +0.28, friction +0.16)
3. Snug is not a press
4. Individual parts, then a linked assembly (stator rigid + plate/cage/roller revolutes; rotor is one piece)
5. Print a thin flat thrust under the plate (large PCD, keeper walls, no 608, no tall drum, no loose bushing)
6. Keep the machine even (3 blades at 120°, 60° helix from 30°)
7. Blades and hub are one part (thin plate, organic roots, tip taper to a flat landing)
8. The section is a 2026-appropriate airfoil
9. Scale is a parameter; 1.0 fills an X2D
10. Print rotational parts lying down (axle is a puck, not a tower)
11. Helical blades, not a straight fence
12. Ship an assembly drawing
13. Publish the design report (iteration + plastic cost)
14. Export one laid-out plate (`01-kit`) in PLA Orange + PLA Glow (cartridge is PIP)

Later (not this exam): catalog metal bearings from a standard table at
larger sizes.
