# noBS CAD knowledge update log

## 2026-08-16

- **Update**: Print-kit rotor is a root plate out to the blades with a
  socket over the bushing flange. Helical lofts start at `plate_z` so
  the airfoil draft ends on that flat sit plane, not from a surface
  above. Open-top bushing: drop the cartridge in, then drop the plate
  on. Tiny ring + skinny-arm hangers are a fail.
- **Update**: Print-kit rotor seats a deck on the bushing shoulder.
  Blade roots are XY airfoils on that deck (print arms + stumps); the
  helical loft starts at `blade_root_z`, not mid-hub. The deck is the
  rotating mount (`hub_mount` + `bushing_spin`). Overhung roots fail
  both print and aero.
- **Update**: Print-kit FDM pass: PIP pockets are +0.80 (2 nozzles), not
  assembled running +0.40. Every bed-printed locate (hub bore, bushing ID,
  axle square, retainer square) has a 0.80 mm elephant-foot lead-in. The
  bushing is not nested around the PIP rollers — that gap is 0.10 mm/side
  and welds. Cage is a spacer; axial capture is flange + retainer.
- **Update**: Print-kit bushing is a distinct outer-race ring with an
  external shoulder. The hub friction-mounts on the bushing OD (`hub_mount`)
  and sits on that seat. Rollers run inside the bushing ID (`bushing_spin`);
  the hub is not the outer race. 12 components / 11 joints at exam scale.
- **Update**: Print-kit is a linked assembly, not a nest that happens to
  spin. Rigid `axle_sit` / `retainer_sit`; revolute `rotor_spin`,
  `cage_spin`, and each `roller_*_spin`. Hub-on-rollers stays running
  (+0.40) — a friction fit would lock the bearing. Axle sits on the
  base. Retainer has a square slip hole. Blades stay one body with the
  hub (centrifugal + cyclic root bending).
- **Update**: Print-kit fits now model 0.20 axial float at every running
  land (base↔flange, flange↔hub/cage, hub↔retainer). The retainer is a
  washer (OD < hub OD). Export is one laid-out `01-kit.3mf` in PLA Orange
  and PLA Glow only — not the assembled nest and not five colored plates.
- **Update**: Print-kit assembly is five parts / five occurrences.
  `assembly_create_component` already inserts the root instance; a
  second `assembly_create_occurrence` duplicated every part. The
  revolute must use on-axis hub/race faces — a spar face at the same Z
  yanked a second rotor off-axis.
- **Update**: Print-kit exam starts from a blank document
  (`cad_new_project`, 0 bodies) and hides construction planes / loft
  sketches before writing `Print-Kit-Tutor.nbcad`. Desktop crash recovery
  of the old tan/red nest is not the current kit — File → New, then Open.
- **Update**: Print-kit exam wipes `Documents/noBS-CAD/Print-Kit-Tutor/`
  before writing plates so retired shaft/hub/wings/plate/bushing/cap
  3MFs cannot sit next to the current five-plate kit.
- **Update**: Print-kit tutor is now a five-part **assembly**: one-piece
  helical rotor, short square stator, flanged inner-race puck, PIP roller
  cartridge, retainer. Role-based fits (running +0.40 / slip +0.28 /
  friction +0.16). Scale parameter (1.0 = Bambu Lab X2D max; exam 0.4).
  Named `assembly_*` tools + A3 drawing. Five print plates.
  Contract: [PRINT_KIT_DESIGN.md](../docs/agentic/PRINT_KIT_DESIGN.md).
- **Update**: MCP now has named `assembly_*` tools (38), `solid_move_copy`,
  `nbcad://assembly` / `nbcad://assembly_solution`, and `assemble_joint` /
  `check_interference` prompts. The print-kit tutor uses components,
  occurrences, and a revolute. Matrix: [MCP_GAP.md](../docs/agentic/MCP_GAP.md).
- **Update**: Print-kit export is seven print-plate 3MFs
  (`Documents/noBS-CAD/Print-Kit-Tutor/`), not one assembled nest.
  This stack is not print-in-place.
- **Update**: Print-kit slim / low-friction pass: Y-frame stand (no
  cookie plate), two-land printed sleeve L/D 1.0 with a mid-groove
  relief, narrow Ø12 thrust land. Fits stay +0.40 — tight FDM binds.
  Contract: [PRINT_KIT_DESIGN.md](../docs/agentic/PRINT_KIT_DESIGN.md).
- **Update**: Print-kit sanity pass: helical NACA 0021 (60° on R24),
  stand shrunk to Ø72 / Ø5 posts so the frame no longer out-girths the
  rotor (plate/rotor 1.42, post/chord 0.31). Straight fence + Ø90 cookie
  fail the new `sanity` and `helix` lessons.
  Contract: [PRINT_KIT_DESIGN.md](../docs/agentic/PRINT_KIT_DESIGN.md).

## 2026-08-15

- **Update**: GD&T / printability study closed the first assembled-spinner
  faults (rotor/post collision, butt-joint posts, through-pocket bushing,
  parallel cones, round-on-round drive, hub swallowing the shoulder).
  Record: [docs/agentic/PRINT_KIT_GDT.md](../docs/agentic/PRINT_KIT_GDT.md).
- **Update**: Replaced the print-bed 608 coupon with a fully printed even
  spinner (assembled stack, 45° cone thrust, printed sleeve, 3+2 even
  layout). Documented the multi-body vs assembly gap in
  [docs/agentic/ASSEMBLY.md](../docs/agentic/ASSEMBLY.md).
- **Update**: Promoted the print-kit tutor to **benchmark #1** in
  [docs/agentic/INTEGRATION_TESTS.md](../docs/agentic/INTEGRATION_TESTS.md)
  (ordered MCP integration tests).
- **Update**: Added `model_print_kit` — a reusable CAD synthesis exam
  (journal + 608 bushing + housing + helical loft) that grades 0.4 mm Bambu
  nozzle tolerancing. Spec `scripts/fixtures/print-kit-tutor.spec.json`;
  rerun `npm run test:mcp-print-kit` or `cargo test print_kit_tutor`.
- **Update**: Added `model_print_tool` — an assistant walkthrough recipe for
  sketching a useful small 3D-printed part (desk cable clip), then fillet /
  chamfer / hole and `print_3mf`.
- **Update**: MCP main surface now advertises focused reads for sketch,
  profiles, features, visibility, appearances, materials, and workspace, plus
  recipes for import/export STEP, profile solids, drawing export, undo, and
  `cad_invoke`. HLR projection and session focus/window templates stay as
  tools, not resources.

## 2026-08-13

- **Update**: MCP harness concept now records recommended protocol
  `2026-07-28` (`server/discover` + `_meta`). `initialize` is a compatibility
  pathway only; the first legacy reply includes the runtime-upgrade manual.
  Live co-link takes a UI-published writer lock; UI heartbeat must not clobber
  MCP revisions (`npm run test:session-bridge`). Switching `cad_attach` to
  another session releases the previous live writer; a failed switch keeps it.
  MCP resources (`nbcad://…`) and prompts cover document/scene/drawing/sessions;
  drawing DTO + Browser visibility tools wrap existing engine methods.

## 2026-07-29

- **Update**: Aligned the bundle with OKF v0.2 and current `main`.
- **Validation**: Added automated structure and internal-link checks.

## 2026-07-28

- **Update**: Aligned concepts with maintainer feedback — goals vs proposals,
  co-link first / multi-window deferred, and agent steering files kept internal.

## 2026-07-27

- **Creation**: Seeded the bundle from the README product stance and MCP docs.
