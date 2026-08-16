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
CAD exam, not a one-off prototype. MCP teaches a 0.4 mm Bambu nozzle
tolerance stack by building a printed VAWT — assembled on one
axis — and grading it.

- Spec: [print-kit-tutor.spec.json](../../scripts/fixtures/print-kit-tutor.spec.json)
- Recipe: `prompts/get model_print_kit`
- Engine exam: `cargo test --manifest-path mcp-server/Cargo.toml print_kit_tutor`
- Agent exam: `npm run test:mcp-print-kit`
- Assembly gap: [ASSEMBLY.md](../../docs/agentic/ASSEMBLY.md)

Clearance is modeled in CAD (+0.40 mm diametral). No FDM press fits. Bodies
are placed in assembly order (base, shaft, hub, three wings, top plate,
printed bushing, cap). The frame is a two-bearing stand. Each wing is a
named symmetric airfoil (NACA 0021) that drops a tenon into a hub socket.
Prefer printed bushings over rollers. The recipe is adversarial
([PRINT_KIT_DESIGN.md](../../docs/agentic/PRINT_KIT_DESIGN.md)); the exam
writes a design report with plastic cost. Fits:
[PRINT_KIT_GDT.md](../../docs/agentic/PRINT_KIT_GDT.md).
Metal 608 bearings are later catalog hardware, not part of this exam.
