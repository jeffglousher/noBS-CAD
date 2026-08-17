---
type: Concept
title: Print-kit tutor
description: Reusable MCP synthesis exam for FDM-tolerant mechanical CAD.
status: stable
updated: 2026-08-16
---

# Print-kit tutor

**Benchmark #1** in the agentic MCP suite
([integration tests](../../docs/agentic/INTEGRATION_TESTS.md)). A rerunnable
CAD exam, not a one-off prototype. MCP teaches role-based 0.4 mm Bambu
nozzle fits by building a printed VAWT **assembly** and grading it.

- Spec: [print-kit-tutor.spec.json](../../scripts/fixtures/print-kit-tutor.spec.json)
- Recipe: `prompts/get model_print_kit`
- Engine exam: `cargo test --manifest-path mcp-server/Cargo.toml print_kit_tutor`
- Agent exam: `npm run test:mcp-print-kit`
- Assembly: [ASSEMBLY.md](../../docs/agentic/ASSEMBLY.md) (named `assembly_*` tools; this exam uses them)
- Domain matrix: [MCP_GAP.md](../../docs/agentic/MCP_GAP.md)

Five functional print parts (base, axle puck, one-piece helical rotor,
PIP roller cartridge, retainer). The CAD assembly links every moving
body: rigid stator (axle sits on the base; retainer on the post), plus
revolutes for the plate bore (`rotor_spin`), cage, and each roller.
There is no loose bushing sandwich and no tall drum. Same-plate PIP
pockets are +0.80; assembled races stay +0.40. Cage height matches
roller height. Bed-printed locates get a 0.80 mm lead-in. The plate is
not nested around the PIP rollers on the bed. The rotor is a **root
plate** (≥5 mm, print sit + upper thrust race). A **thin flat thrust**
under that plate (short pucks on a large PCD; flange = lower race)
takes rotation about Z. Overturning is a couple across that disk — not
a tall journal, not moment webs climbing a wall. Matching 8 mm flats is
still a pancake stack. Look at the solid (`npm run render:print-kit`)
as a check, not as the design method. Blade lofts start on that plate
(flat sit-plane cut — the draft ends on that horizontal, not from a
surface above). Install: drop the cartridge on the flange, then drop
the rotor on the pack. Blades stay one printed body with the plate. The
exam starts from a blank document (`cad_new_project`, 0 bodies) and
hides construction planes before writing the `.nbcad`. Reruns wipe
retired plates so `Print-Kit-Tutor/` holds only `01-kit.3mf` (parts
laid out; PLA Orange + PLA Glow). Fits are per role (running +0.40,
slip +0.28, friction +0.16) with 0.20 axial float at every running
land. Scale 1.0 is Bambu Lab X2D-max; the exam runs at 0.4. No FDM
press fits. No metal 608s. The recipe is adversarial
([PRINT_KIT_DESIGN.md](../../docs/agentic/PRINT_KIT_DESIGN.md)); the exam
writes a design report with plastic cost. Fits:
[PRINT_KIT_GDT.md](../../docs/agentic/PRINT_KIT_GDT.md).
