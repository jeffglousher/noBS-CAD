# Steerable MCP — agent operating rules

## Invariants (do not break)

1. **Disclosure is guidance, not a jail.** Never reject `tools/call` with “not in focus.”
2. **Hard errors** = missing IDs, invalid sketch state, kernel failure only.
3. **Notification name** must stay exactly `notifications/tools/list_changed`.
4. **Stdout** = JSON-RPC only; logs on **stderr**.
5. **Headless goldens** must work without `cad_attach`.
6. **Offline/local** invariant; stdio is current transport, not forever law.
7. When adding a modeling tool: update `tags_for_tool` in `disclosure.rs` **and** the pack matrix test.
8. **MQ pattern:** system-integration / multi-window transport tests must go through
   `nbcad-mcp-bus` (`InMemoryBus` or `bus-jsonl`). Do not “prove” broker routing with
   direct mutex calls. See [mcp-message-bus.md](../mcp-message-bus.md).
9. **Protocol dual-era:** modern clients use `server/discover` + `_meta`
   (`2026-07-28`). Cursor/VS Code still use `initialize` (`2025-06-18`). Do not
   drop the legacy handshake, and do not return `2026-07-28` from `initialize`.

## Modes

| Mode | Who | Behavior |
|------|-----|----------|
| `dynamic` (default) | Main agent / human | Spine ∪ active ∪ soft packs |
| `full_static` | Subagents / broken clients | Advertise all tools |

Prefer `cad_list_all_tools` for planners over leaving the main session in `full_static`.

## Focus packs

`document | sketch | solid | modify | body_ops | datums | history | inspect | print`

Keep `disclosure::tags_for_tool` aligned when adding dialogs or export tools.

## Session bridge (honest scope)

`cad_list_sessions` / `cad_attach` / `cad_refresh` / `cad_detach` under
`NBCAD_SESSION_DIR`:

- session ids are **UUID v8** (BLAKE3, nbcad layout 1); legacy v4 dirs still attach;
- UI publishes `<uuid>/{model,focus,heartbeat,writer}.json` (Tauri or Vite HTTP bridge);
- `mode: "read_only"` (default) — load only; no writeback; no writer claim;
- `mode: "live"` — fresh heartbeat required; MCP claims `writer=mcp` and writebacks
  `model.json` after mutating tools; UI polls `source: "mcp"` generations;
- writer conflict errors when UI holds the lock;
- headless goldens still work without attach.

Installer / UI launch: [#32](https://github.com/jackControls/noBS-CAD/pull/32).


## Print export

Prefer `solid_export_3mf` for slicers. Metadata targets (Bambu/Orca/Prusa/Cura)
are compatible hints — not a full pre-sliced project. STL is geometry-only.

## Related reading

- [mcp-harness.md](../mcp-harness.md)
- Issues [#10](https://github.com/jackControls/noBS-CAD/issues/10), [#11](https://github.com/jackControls/noBS-CAD/issues/11)
- MCP tools / listChanged: https://modelcontextprotocol.io/specification/2026-07-28/server/tools
  (legacy handshake docs: https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
