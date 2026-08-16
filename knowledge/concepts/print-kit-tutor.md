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

Five functional parts (base, axle puck, one-piece helical rotor, PIP
roller cartridge, retainer), then components / occurrences / a revolute
and an A3 drawing. Fits are per role (running +0.40, slip +0.28, friction
+0.16). Scale 1.0 is Bambu Lab X2D-max; the exam runs at 0.4. No FDM
press fits. No metal 608s. The recipe is adversarial
([PRINT_KIT_DESIGN.md](../../docs/agentic/PRINT_KIT_DESIGN.md)); the exam
writes a design report with plastic cost. Fits:
[PRINT_KIT_GDT.md](../../docs/agentic/PRINT_KIT_GDT.md).
