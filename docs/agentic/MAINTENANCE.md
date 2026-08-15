# Maintenance — MCP & disclosure

## Prerequisites (Windows)

```powershell
# After vcpkg install (see docs/WINDOWS_PACKAGING.md):
$env:OCCT_ROOT = "$PWD\vcpkg_installed\x64-windows"
$env:Path = "$PWD\vcpkg_installed\x64-windows\bin;$env:Path"
```

Point MCP clients at the release binary after build:

```text
.../mcp-server/target/release/nbcad-mcp
```

Example Cursor / VS Code config:

```json
{
  "mcpServers": {
    "nbcad": {
      "command": "/absolute/path/to/noBS-CAD/mcp-server/target/release/nbcad-mcp"
    }
  }
}
```

## Tests

```powershell
cargo test --manifest-path mcp-server/Cargo.toml
npm run test:mcp-print-kit
```

CI: `.github/workflows/mcp-server.yml` (Windows + vcpkg OCCT).
Pinned vcpkg checkout must use `fetch-depth: 0` (versioned port trees fail on shallow clones).

## Adding an MCP tool

1. Register `ToolSpec` in `mcp-server/src/main.rs` `tool_specs()` (drawing commands live in `drawing_tools.rs`).
2. Add pack tags in `disclosure::tags_for_tool` (and `auto_focus_for_tool` if needed). `cad_drawing_*` is tagged as the drawing pack by prefix.
3. Update `MODELING_TOOL_COUNT` / pack count assertions if it is a modeling tool.
4. If the tool is a product feature, add it to `mcp-server/src/full_control.rs` and keep `npm run check:mcp-control` green.
5. Add or extend a headless golden under `#[cfg(test)]` in `main.rs`.
6. Update `docs/mcp-harness.md` matrix row if packs change.
7. Run the test suite with OCCT DLLs on `PATH`.

## Disclosure knobs (defaults)

| Knob | Default |
|------|---------|
| Throttle | 300 ms |
| Soft TTL | 60 s |
| Soft LRU | 2 packs |
| Re-promote | 15 s |
| Default focus | `document` |

## Snapshot bridge sessions

- Env: `NBCAD_SESSION_DIR` (else `%TEMP%/nbcad-sessions`)
- Layout: `<uuid>/{model.json,focus.json,heartbeat.json}` (UUID v8 / BLAKE3; legacy v4 accepted)
- Tauri owns one UUID per desktop window and reserves publish generations before async export
- `cad_attach`: read-only load into the MCP process; never writeback; not live co-link
