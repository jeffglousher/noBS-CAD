# MCP playbook (day to day)

Practical notes for driving the **as-built** headless MCP server.

**Recommended protocol:** MCP **2026-07-28**. Start with `server/discover`,
then put `io.modelcontextprotocol/protocolVersion` on every request. Full
recipe: [agentic/MCP_2026.md](agentic/MCP_2026.md). Spec:
https://modelcontextprotocol.io/specification/2026-07-28/

`initialize` (`2025-06-18`) is a compatibility pathway only. If you already
used it, keep modeling — and switch at runtime when you can.

Design / proposals: [mcp-harness.md](mcp-harness.md),
[proposed-architecture.md](proposed-architecture.md).

## Setup (stdio, local)

```sh
cargo build --release --manifest-path mcp-server/Cargo.toml
```

Point the MCP client at:

```text
.../mcp-server/target/release/nbcad-mcp
```

Needs native OCCT (`OCCT_ROOT` when not in a default install). Logs stay on
**stderr**.

## Session habits

1. Keep **one** MCP process for a headless golden or experiment.
2. Read `cad_document` / `solid_scene` before editing.
3. Use stable IDs from scene/status for later ops.
4. Default **`dynamic`** disclosure; call `cad_set_focus` as you model.
5. Subagents: `full_static` or `cad_list_all_tools`.
6. Optional headless session attach: `cad_list_sessions` → `cad_attach`
   (read-only load from `NBCAD_SESSION_DIR`); goldens do not require attach.

Soft disclosure: out-of-focus tools stay **callable**; results may include
`_disclosure`.

## Basic modeling loop

1. `sketch_begin` on a plane
2. Add geometry + constraints
3. `sketch_finish` → `sketch_profiles`
4. `solid_extrude` / other `solid_*` tools
5. Inspect with `solid_scene` / `cad_document`

## Small recipes

Prefer `prompts/get` for the same loops (`model_box`, `model_hole`,
`model_solid`, `print_3mf`, `model_print_tool`, `model_print_kit`, `import_step`, `export_step`,
`drawing_sheet`, `drawing_export`, `undo_history`, `invoke`, `attach_ui`).

| Name | Idea |
|------|------|
| Box | rectangle → extrude → one body |
| Hole | box → hole on a face |
| Solid | profiles → revolve / sweep / loft / rib |
| Printable tool | locked sketches → face features → fillet/chamfer/hole → 3MF (`model_print_tool`) |
| Print kit tutor | **Benchmark #1** — printed turntable (cone/land thrust, printed bushing, platter larger than the foot); `npm run test:mcp-print-kit` (`model_print_kit`). Catalog: [agentic/INTEGRATION_TESTS.md](agentic/INTEGRATION_TESTS.md). Study: [agentic/PRINT_KIT_GDT.md](agentic/PRINT_KIT_GDT.md). Gap: [agentic/ASSEMBLY.md](agentic/ASSEMBLY.md) |
| STEP | `solid_import_step` / `solid_export_step` |
| Drawing | sheet → HLR → DXF / SVG |
| History | `cad_undo` / `cad_redo` / timeline tools |

Read state with `resources/read` (`nbcad://document`, `scene`, `sketch`,
`profiles`, `features`, `drawing`, `workspace`, `visibility`, `appearances`,
`materials`). Tools remain the mutation path.

Print-ready **3MF** with materials/colors: `solid_export_preflight` then
`solid_export_3mf` (slicer Metadata hints, not a sliced G-code project).
**STEP:** `solid_export_step` (AP242 base64). **Import STEP:** `solid_import_step` (base64 exchange bytes). **Drawings:** `cad_drawing_create_sheet` / `cad_drawing_auto_layout` / `cad_drawing_add_*` for commands; `cad_drawing_command` for any `op`; `cad_drawing_document` / `nbcad://drawing` for the DTO; `cad_drawing_project_sheet` for native HLR; `cad_drawing_export_dxf` / `cad_drawing_export_svg` / `cad_drawing_export_profile_dxf` for paper/profile interchange. Live UI: `cad_set_workspace` `drawing` or `solid`. Application history: `cad_undo` / `cad_redo`. Raw engine: `cad_invoke`.

## Failures

Include in issues: tool name, args, last success, error text, OS, and whether
you used UI attach (`cad_attach`) or a headless-only session.
