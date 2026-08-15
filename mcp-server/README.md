# noBS CAD MCP server

`nbcad-mcp` is a native **stdio** JSON-RPC MCP server.

**Recommended protocol: MCP `2026-07-28`.** Call `server/discover`, then send
`params._meta.io.modelcontextprotocol/protocolVersion` (`2026-07-28`) and
`clientCapabilities` on every request. `initialize` is not required. Success
manual: [docs/agentic/MCP_2026.md](../docs/agentic/MCP_2026.md). Spec:
https://modelcontextprotocol.io/specification/2026-07-28/

The older `initialize` handshake (`2025-06-18` and earlier) remains a
**compatibility pathway**. It still works; it is not recommended. That
handshake never returns `2026-07-28`. The first reply to it includes the
runtime-upgrade recipe so agents that can switch on this process can do so.

It covers sketch, solid, print, drawing-command, drawing-document, browser-visibility, application undo/redo, and a mechanical `cad_invoke` / `cad_drawing_command` escape hatch
with **soft focus-scoped disclosure** (`tools.listChanged: true`). Out-of-focus
tools stay callable. Product state is also readable as MCP **resources**
(`nbcad://document`, `nbcad://scene`, `nbcad://sketch`, `nbcad://profiles`,
`nbcad://features`, `nbcad://drawing`, `nbcad://workspace`, …) and recipes as
**prompts** (`model_box`, `model_solid`, `print_3mf`, `model_print_tool`,
`model_print_kit`, `import_step`, `export_step`, `drawing_export`,
`undo_history`, `invoke`, …).

For message-queued systems (Kafka / MQTT / NATS), set
`NBCAD_MCP_TRANSPORT=bus-jsonl` and bridge `BusMessage` frames — see
[docs/mcp-message-bus.md](../docs/mcp-message-bus.md). Stdio remains the
default editor path.

> Notes: [docs/mcp-harness.md](../docs/mcp-harness.md).
> Proposed ideas (in-process co-link, multi-window broker, …):
> [docs/proposed-architecture.md](../docs/proposed-architecture.md).

**Spine controls:** `cad_get_focus`, `cad_set_focus`, `cad_set_workspace`,
`cad_invoke`, disclosure mode get/set, `cad_list_all_tools`,
`cad_cancel_recompute`, `cad_list_sessions`, `cad_attach`, `cad_refresh`,
`cad_detach` (read-only snapshot or live writeback).

**Print:** prefer `solid_export_3mf` (mm + materials + slicer Metadata). Also
`solid_export_stl`, `solid_export_step` (CAD), `material_catalog`,
`body_appearances` / `set_body_appearance`, `solid_export_preflight`, and
`demo_export_pip_3mf`.

**Drawings / Browser:** first-class `cad_drawing_*` command tools (create sheet, auto layout, views, annotations, templates, revisions, BOM, undo/redo) plus `cad_drawing_command` for any `op` and `cad_drawing_document` / `cad_set_drawing_document` for the DTO. Native HLR: `cad_drawing_project_sheet` / `cad_drawing_projection`. Paper interchange: `cad_drawing_export_dxf` / `cad_drawing_export_svg` / `cad_drawing_export_profile_dxf`. Application history: `cad_undo` / `cad_redo`. Live UI workspace: `cad_set_workspace`. File import: `solid_import_step`. `cad_project_visibility` / `cad_set_project_visibility` for hidden bodies/datums/sketches.

## Build and verify

Ordered MCP benchmarks start with the print-kit tutor
([docs/agentic/INTEGRATION_TESTS.md](../docs/agentic/INTEGRATION_TESTS.md) #1):

```sh
cargo build --release --manifest-path mcp-server/Cargo.toml
cargo test --manifest-path mcp-server/Cargo.toml print_kit_tutor
npm run test:mcp-print-kit
cargo test --manifest-path mcp-server/Cargo.toml
```

Windows:

```powershell
$env:OCCT_ROOT = "$PWD\vcpkg_installed\x64-windows"
cargo test --manifest-path mcp-server/Cargo.toml
```

Logs on **stderr**; stdout is JSON-RPC.

Prefer `dynamic` disclosure for main agents; `full_static` or
`cad_list_all_tools` for subagents.

Manual Cursor / VS Code config (build the release binary first):

```json
{
  "mcpServers": {
    "nbcad": {
      "command": "/absolute/path/to/noBS-CAD/mcp-server/target/release/nbcad-mcp"
    }
  }
}
```

Client installer (`install-mcp`) and UI launch tools are follow-ups.

## Modeling flow

The server is stateful. A normal sequence is:

1. `sketch_begin`
2. one or more `sketch_add_*` and constraint tools
3. `sketch_finish`
4. `sketch_profiles`
5. A solid creation tool such as `solid_extrude`, `solid_revolve`,
   `solid_sweep`, `solid_loft`, or `solid_rib`
6. `solid_scene` and `cad_document`

`sketch_begin` accepts a required `plane` object. For a stable planar face,
the optional `face_origin` value can be `face_center` or
`global_origin_projection`; omitting it preserves the support face's kernel
origin for compatibility with existing MCP clients.

After a body exists, use stable edge IDs from `solid_scene` with
`solid_fillet`/`solid_chamfer`, or a planar face ID and one or more face-local
positions with `solid_hole`. Hole positions may carry stable sketch-point
references, and finite holes support flat or angled drill-point bottoms
(118┬░ is the application default). Matching definitions/edit tools preserve
these operations in the same replayable history.

`solid_hole` also accepts optional ISO metric coarse/fine or ASME B1.1
UNC/UNF internal-thread data. Use a common `6H` class for ISO metric or `2B`
for Unified threads unless the design requires another fit. The hole
`diameter` remains the editable predrill diameter; `thread.nominal_diameter`
is the major diameter. `modeled` creates a 60┬░ helical B-rep, while
`simplified` keeps the cylindrical predrill for faster replay and preserves
the complete callout for project and STEP metadata.

Solid calls run the same Rust replay planner and native OCCT adapter as the
desktop application. IDs returned by one call are stable inputs to later
calls in the same feature history.

`sketch_profiles` returns closed profiles plus stable analytic line, arc,
circle, and spline path references. A straight line can be used directly as a
Revolve axis. Connected analytic curves can drive Sweep and guided Loft, and
line/arc/circle/spline entities can drive Rib. Loft accepts an ordered list of
profile references from two or more sketches, optional centerline/guide paths,
and G0/G1/G2 continuity. Rib supports Distance, To Next, Up to Face, and
Through All extents. Every implemented solid family exposes matching
definition/edit tools and supports New Body, Join, Cut, and Intersect where
that operation is meaningful.

Construction-plane tools create and edit Offset, Midplane, and Plane at Angle
features with stable datum IDs. Body-operation tools expose Shell, Mirror,
one/two-direction Rectangular Pattern, Circular Pattern, Combine, and Split
Body through the same replayable history as the interactive application.

`cad_project_model` returns the authoritative versioned `model.json`,
`cad_load_project_model` transactionally restores and recomputes it, and
`cad_new_project` clears to an empty document. Session co-link uses
`cad_list_sessions` / `cad_attach` / `cad_refresh` / `cad_detach` under
`NBCAD_SESSION_DIR` (UUID v8 via BLAKE3; legacy v4 still attaches).
`mode=read_only` loads a snapshot. `mode=live` takes the writer lock from a
UI-published session and writebacks `model.json` after mutating tools. UI
polls `source: "mcp"` revisions; UI heartbeat must not clobber them.
`cad_attach` to another session detaches first so `writer=mcp` is not stranded;
a failed switch keeps the current attach.
Each MCP process still owns one headless document unless attached.
Revisioned MCP→UI sync and installer/UI launch remain follow-ups.

**Print handoff:** `solid_export_preflight` → `set_body_appearance` (optional) →
`solid_export_3mf` (preferred for slicers). Use `solid_tessellate` to inspect
triangle counts before exporting. `demo_export_pip_3mf` returns a built-in
print-in-place demo (AABB clearance smoke ≥ 0.4 mm) without mutating the document.
