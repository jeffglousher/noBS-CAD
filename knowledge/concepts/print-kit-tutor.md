---
type: Concept
title: Print-kit tutor
description: Reusable MCP synthesis exam for FDM-tolerant mechanical CAD.
status: stable
updated: 2026-08-15
---

# Print-kit tutor

**Benchmark #1** in the agentic MCP suite
([integration tests](../../docs/agentic/INTEGRATION_TESTS.md)). A rerunnable
CAD exam, not a one-off prototype. MCP teaches a 0.4 mm Bambu nozzle
tolerance stack by building a fully printed even spinner — assembled on one
axis — and grading it.

- Spec: [print-kit-tutor.spec.json](../../scripts/fixtures/print-kit-tutor.spec.json)
- Recipe: `prompts/get model_print_kit`
- Engine exam: `cargo test --manifest-path mcp-server/Cargo.toml print_kit_tutor`
- Agent exam: `npm run test:mcp-print-kit`
- Assembly gap: [ASSEMBLY.md](../../docs/agentic/ASSEMBLY.md)

Clearance is modeled in CAD (+0.40 mm diametral). No FDM press fits. Bodies
are placed in assembly order (base, shaft, rotor, top plate, printed
bushing, cap). Thrust is a printed 45° cone-in-cup. The rotor is a hub plus
two even buckets, not a lone C coupon. Metal 608 bearings are later catalog
hardware, not part of this exam.
