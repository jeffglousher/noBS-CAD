# Assembly vs multi-body

Jack’s engine on `main` has a real assembly layer: components, occurrences,
joints, positions, motion studies, and approximate interference / swept
collision. Host dispatch is `assembly_*` in `crates/sketch/src/host.rs`.

MCP now wraps **every** one of those methods as a named tool in
`FocusPack::Assembly` (`mcp-server/src/assembly_tools.rs`). Agents should
call `assembly_create_joint`, not `cad_invoke`. The print-kit tutor now
builds **individual parts**, then forms a linked assembly: one component
and one occurrence per moving body, rigid stator joints, a rigid hub
mount on the bushing OD, and revolutes for the bushing, cage, and each
roller. `assembly_create_component` already inserts the root
occurrence — a second `assembly_create_occurrence` duplicates every
part and the solver will yank a copy off-axis. Joint connectors must
be on-axis circular edges (bushing ID / OD, hub bore, axle race), not a
blade-spar face — planar face centroids yank the rotor off-axis. Fits
stay numbers (running / slip / friction) — joints do not invent metal
608s. The hub is not the outer race.

## What the engine and MCP both have

- Component definitions and occurrence tree
- Joints (rigid, revolute, slider, cylindrical, planar, ball, pin_slot, screw, universal)
- Joint motion, mechanism drag, grounded bodies
- Saved positions and motion studies
- Approximate interference and swept-collision reports
- `nbcad://assembly` and `nbcad://assembly_solution`
- Recipes: `assemble_joint`, `check_interference`
- Solid Move/Copy as `solid_move_copy` / `solid_edit_move_copy`

`cad_set_workspace assembly` advertises the assembly pack. `cad_invoke`
remains the escape hatch for host methods that land before a named wrapper.

## What the print-kit tutor must not do

- Do not skip the assembly step (components + occurrences + a revolute)
- Fits stay numbers (running +0.40 / slip +0.28 / friction +0.16), not
  a substitute for mates and not a press
- Until catalog hardware exists, kits **print every bearing surface**
  (flange thrust land, PIP roller cartridge). No hidden 608s
- Engine joints do not invent metal bearings or screws
- Do not export the assembled nest as the print job

## Multi-body without mates (still valid)

- `solid_*` `operation: "new_body"` — independent bodies in one document
- `solid_move_copy` for placement after the body exists
- Per-body appearance / 3MF materials
- Combine, split, mirror, rectangular / circular pattern
- Manual placement by sketching on datums

See [MCP_GAP.md](MCP_GAP.md) for the full pack matrix.
