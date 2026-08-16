# Print-kit tutor (CAD synthesis exam)

**Benchmark #1** in the agentic suite
([INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)). This is the reusable
integration test for **AI → fully printable mechanical CAD**. It is the
curriculum, the worked example, and the grader.

The first kits failed that bar: a print-bed scatter, an assembled spinner
that collided, leftover helical C-buckets, a concentric hoop, a turntable
that threw the frame away, then a flat plate sold as a wing. Later kits
used one clearance for every hole, tenoned three wings into a hub, and
stood a tall skinny shaft in a two-land sleeve that could not take tip
moment. Contract: [PRINT_KIT_DESIGN.md](PRINT_KIT_DESIGN.md). Fits:
[PRINT_KIT_GDT.md](PRINT_KIT_GDT.md). Agents start with
`prompts/get model_print_kit` — that recipe is adversarial.

The exam now **does** form a CAD assembly (components, occurrences, a
revolute). See [ASSEMBLY.md](ASSEMBLY.md). It starts with
`cad_new_project` and fails if the scene is not empty — do not continue
a recovered or older Print Kit Tutor. Construction planes and finished
loft sketches are hidden before the `.nbcad` is written so File → Open
shows the five-part kit, not orange datum stacks.

## What to build

A five-part **printed VAWT assembly** (spec
`scripts/fixtures/print-kit-tutor.spec.json`, id `fdm-print-vawt`).
Linear numbers are the Bambu Lab X2D-max design (256×256×260, 8 mm
margin). `spec.scale` shrinks the source (exam default **0.4**). Feature
floors (roller Ø8, TE 0.8, 4-nozzle walls) are clamped.

| Part | Role | How it mates |
|------|------|----------------|
| Base | Y-frame + short square stator post, one piece | Post is the grounded axis. Print flat. |
| Axle | Flanged inner-race puck, square bore | Friction on the post (+0.16). Hub sits on the flange land (0.20 float). Print on the flange. |
| Rotor | Hub + 3 helical **NACA 0021** blades, **one body**, open drafted tips | Hub bore is the outer race. Freewheels on the rollers. Print standing on the hub. |
| Roller cartridge | Cage + 6 PIP rollers, min Ø8, large PCD | Running +0.40 on rollers/races. One print plate. |
| Retainer | Flat ring | Slip +0.28 on the post. Keeps the hub on the land. |

Fits are **per role** (running / slip / friction). Do not use +0.40 on
every hole. Slicer XY hole compensation stays 0. No metal 608s. No FDM
press fits.

Assembly order: **base → axle → roller cartridge → rotor → retainer**.

Then: one `assembly_create_component` per part (that call already
inserts the root occurrence — a second `assembly_create_occurrence`
duplicates every part). Ground the base. Revolute on the **axis**
(hub/race faces, not a blade spar). Ship an A3 assembly drawing with notes.

Print each functional part in its own orientation. The cartridge is
print-in-place. The exam **wipes** `%USERPROFILE%\Documents\noBS-CAD\Print-Kit-Tutor\`
first (retired `02-shaft` / `03-hub` / `04-wings` / `05-plate` /
`06-bushing` / `07-cap` and any assembled `Print-Kit-Tutor.3mf`), then
writes the five current plates and a design report next to the project.

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
**File → Open** the path above. The current kit is a black Y-frame,
jade-white axle puck, green one-piece rotor, orange rollers, red
retainer ring.

Agents start with `prompts/get model_print_kit`.

## Lessons the grader checks

1. Start from a blank document (`cad_new_project`, 0 bodies; hide datums)
2. Fits are per role and material (running +0.40, slip +0.28, friction +0.16)
3. Snug is not a press
4. Individual parts, then an assembly (5 components, 5 occurrences; rotor is one piece)
5. Print a roller pack that takes tip moment (large PCD, no 608)
6. Keep the machine even (3 blades at 120°, 60° helix from 30°)
7. Blades and hub are one part
8. The section is a 2026-appropriate airfoil
9. Scale is a parameter; 1.0 fills an X2D
10. Print rotational parts lying down (axle is a puck, not a tower)
11. Helical blades, not a straight fence
12. Ship an assembly drawing
13. Publish the design report (iteration + plastic cost)
14. Export one plate per functional part (cartridge is PIP)

Later (not this exam): catalog metal bearings from a standard table at
larger sizes.
