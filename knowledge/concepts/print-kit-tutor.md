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

Six functional print parts (base, axle puck, outer-race bushing, one-piece
helical rotor, PIP roller cartridge, retainer). The CAD assembly links
every moving body: rigid stator (axle sits on the base; retainer on the
post), rigid `hub_mount` (hub on bushing OD), plus revolutes for the
bushing, cage, and each roller. The hub is not the outer race.
Hub-on-rollers is a running fit, not friction. Same-plate PIP pockets
are +0.80; assembled races stay +0.40. Bed-printed locates get a 0.80 mm
lead-in. The bushing is not nested around the PIP rollers. The rotor is a
**root plate** out to the blades with a **socket** over the bushing
flange. Blade lofts start on that plate (flat sit-plane cut — the draft
ends on that horizontal, not from a surface above). The bushing is an
open-top cup: drop the cartridge in, then drop the plate on. Blades
stay one printed body with the plate. The exam starts from a blank document (`cad_new_project`,
0 bodies) and hides construction planes before writing the `.nbcad`.
Reruns wipe retired plates so `Print-Kit-Tutor/` holds only
`01-kit.3mf` (parts laid out; PLA Orange + PLA Glow). Fits are per role
(running +0.40, slip +0.28, friction +0.16) with 0.20 axial float at
every running land. Scale 1.0 is Bambu Lab X2D-max; the exam runs at 0.4. No FDM
press fits. No metal 608s. The recipe is adversarial
([PRINT_KIT_DESIGN.md](../../docs/agentic/PRINT_KIT_DESIGN.md)); the exam
writes a design report with plastic cost. Fits:
[PRINT_KIT_GDT.md](../../docs/agentic/PRINT_KIT_GDT.md).
