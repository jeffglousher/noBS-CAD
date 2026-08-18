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
9. **Protocol: recommend 2026-07-28 only.** `server/discover` + per-request
   `_meta`. The `initialize` handshake remains a compatibility pathway — never
   a recommendation. The first reply to a legacy handshake (or the first
   legacy `tools/call`) must include the runtime-upgrade success manual. Do
   not reject legacy calls. Do not return `2026-07-28` from `initialize`.
   Manual: [MCP_2026.md](MCP_2026.md).

## Modes

| Mode | Who | Behavior |
|------|-----|----------|
| `dynamic` (default) | Main agent / human | Spine ∪ active ∪ soft packs |
| `full_static` | Subagents / broken clients | Advertise all tools |

Spine always includes `cad_agent_guidance`, `solid_check`,
assembly entry/motion/interference, and `cad_drawing_create_sheet`.
A new process starts in **sketch** with **solid** soft.
Call `cad_agent_guidance` first and after `cad_set_focus`.
`subscriptions/listen` is the 2026-07-28 opt-in for list_changed;
stdio acks then still pushes the notification.

Prefer `cad_list_all_tools` for planners over leaving the main session in `full_static`.

## Focus packs

`document | sketch | solid | modify | body_ops | datums | history | inspect | print | drawing | assembly`

Definition catalogs live with their feature packs (solid / modify / body_ops).
`cad_undo` / `cad_redo` stay on the spine. Soft LRU is 3. Full matrix:
[MCP_GAP.md](MCP_GAP.md).

Keep `disclosure::tags_for_tool` aligned when adding dialogs or export tools.
When adding a product surface, also expose it as an MCP **resource** when it is
readable state (`nbcad://…`) and a **prompt** when there is a recipe.

## Session bridge (honest scope)

`cad_list_sessions` / `cad_attach` / `cad_refresh` / `cad_detach` under
`NBCAD_SESSION_DIR`:

- session ids are **UUID v8** (BLAKE3, nbcad layout 1); legacy v4 dirs still attach;
- UI publishes `<uuid>/{model.json,active-sketch.json?,focus,heartbeat,writer,window}.json` (Tauri or Vite HTTP bridge);
- while a sketch transaction is open, `model.json` stays at the last completed export and `active-sketch.json` carries the live DTO;
- `mode: "read_only"` (default) — load only; no writeback; no writer claim;
- `mode: "live"` — fresh heartbeat required; MCP **takes** the lock from `ui`/`none` and writebacks
  `model.json` after mutating tools; UI polls `source: "mcp"` generations and must not clobber them;
- writer conflict errors when UI holds the lock after attach; `cad_refresh` loads the UI model without stealing;
- `cad_attach` to another session detaches first (releases a live writer lock); a failed switch keeps the current attach;
- headless goldens still work without attach.

Installer / UI launch: [#32](https://github.com/jackControls/noBS-CAD/pull/32).


## Print export

Prefer `solid_export_3mf` for slicers. Metadata targets (Bambu/Orca/Prusa/Cura)
are compatible hints — not a full pre-sliced project. STL is geometry-only.

## Related reading

- [mcp-harness.md](../mcp-harness.md)
- Issues [#10](https://github.com/jackControls/noBS-CAD/issues/10), [#11](https://github.com/jackControls/noBS-CAD/issues/11)
- MCP 2026-07-28 success manual: [MCP_2026.md](MCP_2026.md)
- MCP tools / listChanged: https://modelcontextprotocol.io/specification/2026-07-28/server/tools
