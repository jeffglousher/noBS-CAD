# Assembly vs multi-body

Jack’s engine on `main` has a real assembly layer: components, occurrences,
joints, positions, motion studies, and approximate interference / swept
collision. Host dispatch is `assembly_*` in `crates/sketch/src/host.rs`.

MCP now wraps **every** one of those methods as a named tool in
`FocusPack::Assembly` (`mcp-server/src/assembly_tools.rs`). Agents should
call `assembly_create_joint`, not `cad_invoke`. The print-kit tutor still
builds a **multi-body part** on one axis with numeric fits — no mates.

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

- Do not add joints, occurrences, or motion studies to the VAWT exam
- Fits stay numbers (`+0.40`), not mates
- Until catalog hardware exists, kits **print every bearing surface**
  (cone thrust, sleeve bushing, printed rollers)
- Engine joints do not invent metal 608s or screws

## Multi-body without mates (still valid)

- `solid_*` `operation: "new_body"` — independent bodies in one document
- `solid_move_copy` for placement after the body exists
- Per-body appearance / 3MF materials
- Combine, split, mirror, rectangular / circular pattern
- Manual placement by sketching on datums

See [MCP_GAP.md](MCP_GAP.md) for the full pack matrix.
