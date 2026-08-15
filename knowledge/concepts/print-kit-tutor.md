---
type: Concept
title: Print-kit tutor
description: Reusable MCP synthesis exam for FDM-tolerant mechanical CAD.
status: stable
updated: 2026-08-15
---

# Print-kit tutor

A rerunnable CAD exam, not a one-off prototype. MCP teaches a 0.4 mm Bambu
nozzle tolerance stack by building a real printable journal kit and grading it.

- Spec: [print-kit-tutor.spec.json](../../scripts/fixtures/print-kit-tutor.spec.json)
- Recipe: `prompts/get model_print_kit`
- Engine exam: `cargo test --manifest-path mcp-server/Cargo.toml print_kit_tutor`
- Agent exam: `npm run test:mcp-print-kit`

Clearance is modeled in CAD (+0.40 mm diametral). No FDM press fits. Every
body sits on z=0. Functional holes are XY circles. The kit includes a 608
envelope bushing and a helical loft so the answer is a manufacturable
assembly, not a single extrusion.
