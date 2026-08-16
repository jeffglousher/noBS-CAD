<!--
  File this on the fork after enabling Issues:

  gh repo edit jeffglousher/noBS-CAD --enable-issues
  gh issue create --repo jeffglousher/noBS-CAD \
    --title "[feat] Resume: MCP CadServer tests and remaining surfaces" \
    --label enhancement \
    --body-file docs/agentic/RESUME_REQUEST.md
-->

## Area

MCP/automation

## Problem

The Linux cloud agent cannot compile or run `nbcad-mcp` (`CadServer::new()`). There is no OCCT 7.9.x SDK and no `gdk-3.0`. Mechanical tools and the main MCP 2026 resource/prompt surface are already on the branch, but native goldens have not run. A new session on a PC with OCCT must pick this up.

## Proposal

Resume Jeff-owned MCP work on fork `jeffglousher/noBS-CAD`. Do not start a new cloud agent for OCCT tests.

**Checkout:** `cursor/mcp-message-bus-60a6` at `227f85bb06447e4d589d168e640dc153991c5975` (`feat(mcp): expose main product state as resources and recipes`), or later commits on the same branch. Stacked on `cursor/sync-mcp-ci-60a6`. Do not leave this branch unless asked. `gh` defaults to upstream `jackControls/noBS-CAD`; fork PRs/issues are `--repo jeffglousher/noBS-CAD`. Preferred base for this stack is `cursor/sync-mcp-ci-60a6`, not `main`.

**Ownership — Jeff:** `mcp-server/`, session bridge, MCP bus, IDs, drawing *command* wrappers, engine host `drawing_command`, MCP-native drawing export (DXF/SVG/profile), resources (`nbcad://…`), prompts.

**Ownership — Jack (do not rewrite):** Bevy viewport, 2D drawings HLR/DXF (`src/drawing/dxf.ts`, OCCT HLR), wasm release opts, desktop packaging CI, ADR 0006, pointer-driven UI helpers in `src/drawing/document.ts`.

**Do not:** port Jack’s DXF writer into Rust; add collaboration-comment MCP tools; invent extra modeling tools unless a surface read/recipe requires one; flip the existing open PR back to draft.

### Already shipped

Tools (mutation path; do not change counts unless you add a real tool):

- `MODELING_TOOL_COUNT = 166`, `PRINT_HELPER_COUNT = 8`, `CONTROL_TOOL_COUNT = 12`, total **186**
- Drawing pack **55** = 2 DTO + 45 commands + `cad_drawing_command` + 2 projection + 2 history + 3 export
- Mechanical: `cad_invoke` (any `host.rs` method; `solid_prepare_*` / `project_prepare_*` replay; `solid_commit` rejected), `cad_drawing_command`, `cad_undo` / `cad_redo`, `solid_import_step`, `cad_set_workspace`, MCP-native DXF/SVG/profile export
- Completeness: `scripts/check-mcp-full-control.mjs` + `npm run check:mcp-control`; `mcp-server/src/full_control.rs`; Windows CI runs the gate before rustfmt/cargo test

Surfaces (MCP 2026 read + recipe path):

- Resources: `nbcad://document|project|scene|drawing|focus|workspace|sessions|sketch|sketches|profiles|visibility|appearances|materials|features` plus template `nbcad://session/{session_id}`
- `nbcad://focus` includes `workspace` (same as `cad_get_focus`)
- New reads assemble from `SketchManager` / catalog JSON (not `call_tool`)
- Prompts: `model_box`, `model_hole`, `model_solid`, `attach_ui`, `print_3mf`, `model_print_tool`, `model_print_kit`, `import_step`, `export_step`, `drawing_read`, `drawing_sheet`, `drawing_export`, `undo_history`, `invoke`
- Constants `MAIN_RESOURCE_URIS` (14) and `MAIN_PROMPT_NAMES` (14) in `mcp-server/src/surfaces.rs` are the catalog source of truth
- Print-kit tutor (**benchmark #1**, fully printed even spinner): [INTEGRATION_TESTS.md](INTEGRATION_TESTS.md), [PRINT_KIT_GDT.md](PRINT_KIT_GDT.md), [ASSEMBLY.md](ASSEMBLY.md), `scripts/fixtures/print-kit-tutor.spec.json`, `npm run test:mcp-print-kit`, `cargo test print_kit_tutor`

**Intentionally not MCP:** theme/6DOF settings, pointer Select, project tab close, collaboration comments, `window.print()` (MCP returns SVG), Jack’s annotation-rich UI DXF writer, disabled ribbon placeholders, #11 in-process shared engine, #12 two UI windows.

### Remaining surface gaps (after CadServer is green)

- `nbcad://projection` (HLR is expensive; keep `cad_drawing_project_sheet`)
- `nbcad://tools` (use `cad_list_all_tools`)
- `nbcad://session/{id}/focus` and `…/window` (`cad_attach` already loads `focus.json`)
- Dedicated construction-plane / body-ops / fillet prompts (tools exist; `model_solid` + `invoke` cover the loop)
- `subscriptions/listen`

## Acceptance criteria

- [ ] `npm run check:mcp-control` reports `102 host methods, 45 drawing ops, 98 ribbon mappings, 11 file features, 14 resources, 13 prompts`
- [ ] `cargo fmt --manifest-path mcp-server/Cargo.toml -- --check` is clean
- [ ] `cargo test --manifest-path mcp-server/Cargo.toml` passes on a machine with native OCCT 7.9.x (`resources_and_prompts_cover_product_surfaces`, `read_product_resource` for the new URIs, invoke/undo goldens, registry counts 166 / 55 / 186)
- [ ] `cargo test --manifest-path crates/sketch/Cargo.toml --lib` — 105 passed
- [ ] `npx tsc --noEmit` && `npm run check:ids` && `npm run check:knowledge` && `npm run test:session-bridge`
- [ ] If Windows `mcp-tests` fails: rustfmt and E0716 temporary-borrow in `surfaces.rs` have bitten this branch before (`prompt_title` must return `String`; bind `list_resources()` / `list_prompts()` before borrowing). Fix on the same branch.
- [ ] After CadServer is green, continue only the next remaining surface gap if asked. Do not add comment tools. Do not port Jack’s DXF writer. Do not bump `MODELING_TOOL_COUNT` unless you add a real modeling tool.

## Resume commands

```sh
git fetch origin
git checkout cursor/mcp-message-bus-60a6
git pull origin cursor/mcp-message-bus-60a6
# expect 227f85bb06447e4d589d168e640dc153991c5975 or later on this branch
cargo test --manifest-path mcp-server/Cargo.toml
```
