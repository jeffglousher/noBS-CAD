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

## What MCP still does not have (named tools)

All 36 `assembly_*` host methods are reachable today via `cad_invoke`
(`method` + `arguments`). There is **no** named `assembly_*` MCP pack
and no `FocusPack::Assembly`. Tool count stays 186 (166+8+12).

| Engine method | Named MCP tool | Why it matters |
|---------------|----------------|----------------|
| `assembly_document` / `assembly_solution` | `cad_invoke` only | Agents should read the occurrence tree without guessing the host name |
| Create/update component or occurrence | `cad_invoke` only | Instance reuse is not in the exam spine |
| Create/update/delete joint | `cad_invoke` only | Fits stay numbers in the print-kit tutor |
| Motion / positions / contact sets | `cad_invoke` only | No kinematics recipe yet |
| Interference / swept collision | `cad_invoke` only | No assemble-check lesson in the tutor |
| Ribbon `bodyFeature:move_copy` | `cad_invoke` only | Jack’s solid move/copy is not a named `solid_*` tool |

Do **not** bump `MODELING_TOOL_COUNT` to paper over this. A later pack
can wrap the existing host methods without rewriting Jack’s crate.
`cad_set_workspace` accepts `assembly` so the live UI can follow.

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
