---
type: Concept
title: Print-kit tutor
description: Reusable MCP synthesis exam for FDM-tolerant mechanical CAD.
status: stable
updated: 2026-08-17
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

One stator (thin Y-frame + race ring + keeper walls + U-window
fence + constant journal), one-piece helical rotor, eight barrel-crowned
radial-axis rollers, clocked E-clip. The CAD assembly grounds the stator, revolutes
the plate bore (`rotor_spin`) and each roller about its **radial**
axis, and sits the clip in the journal groove. There is no loose
bushing sandwich, no tall drum, and no standing-Z puck pack. Assembled
races stay +0.40 because rollers print standing as other bodies.
Top-load slots add two nozzles. Keeper walls survive the cut so
rollers cannot slide into the Y-frame. Bed-printed locates get a
0.80 mm lead-in. The plate is not nested around the rollers on the bed.
The rotor is a **thin root plate** (floor 3.2 mm, print sit + upper
thrust race) with the airfoil through that plate, an **organic blend**
on the top face, and a short **tip taper to a flat landing**. A
**thin flat thrust** under the **blade roots** (radial-axis rollers on a
large PCD; race ring = lower land) takes rotation about Z. Overturning
is a couple under those roots — not a tall journal, not an inboard pack
that leaves the plate as a cantilever, not moment webs climbing a wall.
The fence is a spacer (ID looser than the plate bore). Assemble:
crowned rollers into the U-windows, drop the rotor over the journal, snap the
E-clip. Clip and pack CAD are this pass; the research contract is
[PRINT_KIT_BEARING.md](../../docs/agentic/PRINT_KIT_BEARING.md). Matching 8 mm
flats is still a pancake stack. Standing-Z pucks slide on their end
faces. Look at the solid (`npm run render:print-kit`) as a check, not
as the design method. Blade lofts start on that plate (flat sit-plane
cut — the draft ends on that horizontal, not from a surface above).
Blades stay one printed body with the plate. The exam starts from a
blank document (`cad_new_project`, 0 bodies) and hides construction
planes before writing the `.nbcad`. Reruns wipe retired plates so
`Print-Kit-Tutor/` holds only `01-kit.3mf` (parts laid out; PLA Orange
+ PLA Glow). Fits are per role (running +0.40, slip +0.28, friction
+0.16) with 0.20 axial float at every running land. Scale 1.0 is Bambu
Lab X2D-max; the exam runs at 0.4. No FDM press fits. No metal 608s.
The recipe is adversarial
([PRINT_KIT_DESIGN.md](../../docs/agentic/PRINT_KIT_DESIGN.md)); the exam
writes a design report with plastic cost.
