# MCP domain coverage

Named tools must exist for every **shipped product surface**. Disclosure
packs are guidance, not a jail: out-of-focus tools stay callable.
`cad_invoke` is the last-resort escape hatch, not the assembly API.

Print helpers stay outside `MODELING_TOOL_COUNT`. The print-kit tutor
**does** form joints (`rotor_spin`, roller revolutes, `retainer_sit`).

## Registry (as-built)

| Bucket | Count | Notes |
|--------|------:|-------|
| Modeling | **206** | 166 prior + 38 `assembly_*` + 2 move/copy |
| Print helpers | 9 | Outside modeling (includes `solid_check`) |
| Control | **13** | Focus, workspace, sessions, disclosure, `cad_agent_guidance` |
| **Total** | **228** | |

## Pack matrix (modeling; print excluded)

| Pack | Count | Contents |
|------|------:|----------|
| document | 8 | Name, project load/export, visibility |
| sketch | 50 | One pack this pass — do not split |
| solid | 15 | Creators + extrude/revolve/sweep/loft/rib definition catalogs |
| modify | 9 | Fillet/chamfer/hole + their catalogs |
| body_ops | 16 | Shell, **move/copy**, mirror, patterns, combine, split, STEP import, body-feature catalog |
| datums | ≥6 | Construction planes |
| history | ≥5 | Timeline edits; `cad_undo` / `cad_redo` stay History pack **and** spine |
| inspect | 4 | `solid_scene` / `solid_recompute` / `solid_check` (spine), `solid_tessellate` |
| drawing | 55 | Commands, HLR, DXF/SVG/profile |
| assembly | 38 | Every `assembly_*` host method |

Soft LRU is **3**. A new process starts in **sketch** with **solid**
soft. Spine always includes `cad_agent_guidance`, `cad_set_focus`,
`solid_check`, assembly document/solution **plus**
`assembly_create_component` / `assembly_create_joint` /
`assembly_set_joint_motion` / `assembly_evaluate_motion_study` /
`assembly_interference_check`, `cad_drawing_document` /
`cad_drawing_create_sheet`, `cad_invoke`, `solid_scene`,
`solid_recompute`, and application undo/redo.

`cad_set_workspace assembly` sets `FocusPack::Assembly`.

## Named tools that closed the gap

| Product surface | Named MCP tool | Host / payload |
|-----------------|----------------|----------------|
| Assembly document | `assembly_document` | `assembly_document` |
| Assembly solution | `assembly_solution` | `assembly_solution` |
| Components / occurrences | `assembly_create_*` / `assembly_update_*` / `assembly_duplicate_occurrence` | matching host methods |
| Joints (9 kinds) | `assembly_create_joint` and preview/update/delete/motion | matching host methods |
| Positions / motion studies / contacts | `assembly_*_position` / `assembly_*_motion_study` / `assembly_*_contact_set` | matching host methods |
| Interference / swept collision | `assembly_interference_check` / `assembly_swept_collision_check` | matching host methods |
| Ribbon `bodyFeature:move_copy` | `solid_move_copy` / `solid_edit_move_copy` | `Payload::BodyFeature("move_copy")` |
| Ribbon `joint` | `assembly_create_joint` | host `assembly_create_joint` |
| Ribbon `assemblyWorkspace` | `cad_set_workspace` | focus pack Assembly |

Joint kinds already in `crates/assembly`: rigid, revolute, slider,
cylindrical, planar, ball, pin_slot, screw, universal.

## Resources and prompts

| URI / prompt | Role |
|--------------|------|
| `nbcad://assembly` | Components, occurrences, joints, positions, studies, contacts |
| `nbcad://assembly_solution` | Forward-kinematics poses (underscore so `resourceKindName` stays `AssemblySolution`) |
| `nbcad://guidance` | Static packs + spine + tutor recipes (live next-steps: `cad_agent_guidance`) |
| `assemble_joint` | Recipe: component (inserts root occurrence) → named joint |
| `check_interference` | Recipe: approximate pairwise / motion collision |
| `tutor_exam` | How to author a gold-path MCP exam |

## Intentionally not MCP

- Disabled ribbon placeholders (ellipse, draft, …)
- Viewport / 6DOF / space mouse
- Measure / section overlays
- Multi-tab sessions
- `solid_commit` (rejected)
- Catalog hardware (metal 608s, screws) — print the bearing surface

## Verify

```powershell
$env:OCCT_ROOT = "$PWD\vcpkg_installed\x64-windows"
$env:Path = "$env:OCCT_ROOT\bin;$env:Path"
cargo test --manifest-path mcp-server/Cargo.toml print_kit
cargo test --manifest-path mcp-server/Cargo.toml full_control
npm run test:mcp-print-kit
npm run check:mcp-control
npm run check:knowledge
```
