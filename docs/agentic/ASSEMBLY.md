# Assembly gap

Jack’s engine on `main` now has a real assembly layer: components,
occurrences, joints, positions, motion studies, and approximate
interference / swept-collision checks. The host dispatch is
`assembly_*` in `crates/sketch/src/host.rs`.

The MCP process still does **not** wrap those methods. Agents can only
build a **multi-body part** with `solid_*`. The print-kit tutor stays
honest about that: nine printable VAWT bodies on one axis, no mates.

## What the engine has (desktop / wasm)

- Component definitions and occurrence tree
- Joints, joint motion, mechanism drag, grounded bodies
- Saved positions and motion studies
- Approximate interference and swept-collision reports
- Browser / viewport consume `assembly_solution` poses

## What MCP still does not have

| Missing MCP surface | Why it matters |
|---------------------|----------------|
| `assembly_document` / `assembly_solution` | Agents cannot read the occurrence tree or solved poses |
| Create/update component or occurrence | No instance reuse from MCP |
| Create/update/delete joint | Fits stay numbers in sketches |
| Motion / positions / contact sets | No kinematics from the exam harness |
| Interference / swept collision | No assemble-check tool for the print kit |

Do **not** bump `MODELING_TOOL_COUNT` to paper over this. A later pack
can wrap the existing host methods without rewriting Jack’s crate.

## What MCP can do today

- `solid_*` `operation: "new_body"` — independent bodies in one document
- Per-body appearance / 3MF materials
- Combine, split, mirror, rectangular / circular pattern
- Manual placement by sketching on datums (no `solid_move`)

## Fully printable first

Until catalog hardware exists, kits must **print every bearing surface**
they need (cone thrust, sleeve bushing, printed rollers). Do not design
a mechanism that only works if a metal bearing appears off camera.
The engine joints do not invent hardware.
