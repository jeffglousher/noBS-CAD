# MCP harness notes

How agents and tests can drive noBS CAD **locally** through MCP.
This page separates **what exists today** from **proposed** architecture.
Proposals: [proposed-architecture.md](proposed-architecture.md).
Product directions: [goals.md](goals.md).

## Why MCP
MCP gives coding agents a tool API without turning noBS CAD into a cloud
service. The goal is a **strong local automation** surface for testing and
agent-driven modeling.
**Invariant:** no required cloud control plane. Automation stays on the user's
machine (or CI runner).

## Today (as-built on this branch)
| Topic | Current state |
|-------|----------------|
| Transport | **stdio** JSON-RPC (`nbcad-mcp`) — logs on **stderr**; optional **`bus-jsonl`** envelope for Kafka/MQTT/NATS connectors ([mcp-message-bus.md](mcp-message-bus.md)) |
| Protocol | **Dual-era:** `server/discover` + per-request `_meta` (`2026-07-28`); Cursor/VS Code still `initialize` (`2025-06-18`). Unsupported versions return JSON-RPC `-32022`. |
| Tools | **105** modeling tools + control/export helpers |
| Disclosure | Soft focus-scoped; `tools.listChanged: true`; ~300 ms throttle |
| Notify worker | Stdin reader thread + timed wake — `list_changed` / soft-TTL flush **without** a later client ping |
| Document | One persistent feature history **per MCP process** |
| Sessions | Snapshot + **live** co-link: `cad_list_sessions` / `cad_attach` (`mode`: `read_only`\|`live`) / `cad_refresh` / `cad_detach` |
| Geometry | Same native OCCT replay path as desktop when OCCT is available |
| Export | STEP + STL + **3MF** (`solid_export_*`, `material_catalog`); 3MF preferred for slicers |

### Soft disclosure (not a jail)
Spine → active pack → soft packs (60 s TTL, LRU 2). Hidden tools stay
**callable**; results include `_disclosure`. Escape hatch: `full_static` or
`cad_list_all_tools`. Prefer `dynamic` for main agents.

### Focus packs
```text
document | sketch | solid | modify | body_ops | datums | history | inspect | print
```
Tags: `mcp-server/src/disclosure.rs` (`tags_for_tool`).

### Session bridge (read-only snapshot + live co-link)
Headless goldens work **without** attach.
UI publishes under:
`<NBCAD_SESSION_DIR>/<uuid>/{model.json,focus.json,heartbeat.json,writer.json}`
(atomic writes, generation-guarded). Session ids are **UUID v8** (BLAKE3, nbcad
layout 1); legacy v4 directories still attach. Document names are rejected.

- **Desktop (Tauri):** native `mcp_session_bridge_*` commands.
- **Browser/WASM (Vite):** `/__nbcad_session/*` middleware (`scripts/session-http-bridge.mjs`).

Attach modes:
1. `cad_list_sessions` — UUID dirs; heartbeat `age_ms` / `stale`; `writer` lock metadata.
2. `cad_attach` with `mode: "read_only"` (default) — load snapshot; **no** writeback; does not claim `writer.json`.
3. `cad_attach` with `mode: "live"` — requires a **fresh** heartbeat; claims `writer=mcp`; after mutating tools, MCP writebacks `model.json` + bumps generation (`source: "mcp"`). UI polls and applies.
4. Writer conflict: if `writer.json` is `ui`, MCP mutates fail with a clear error until `cad_refresh` / UI releases the lock.
5. `cad_refresh` / `cad_detach` — re-read or clear attach; live detach releases the writer lock.

Smoke: `npm run smoke:colink` (browser publish → simulated MCP revision → UI apply).
Installer / UI launch: [#32](https://github.com/jackControls/noBS-CAD/pull/32).
Build and tool flow: [mcp-server/README.md](../mcp-server/README.md).
Day-to-day playbook: [agent-mcp.md](agent-mcp.md).

### Stdio (current supported path)
Agents and CI spawn `nbcad-mcp` as an MCP stdio server. One process owns one
document. Prefer `solid_export_3mf` for slicer handoff; STEP for CAD interchange.

Modern clients **SHOULD** probe with `server/discover` (no session handshake).
Cursor and VS Code still send `initialize`; that path stays on `2025-06-18` so
those editors keep working. Do not treat `initialize` returning `2026-07-28` as
success — the spec retired that handshake.

### Disclosure notify behavior
Focus / mode / soft-TTL changes schedule `notifications/tools/list_changed`.
The server wakes on that deadline even if the client is idle — it does **not**
require a later `ping` or tool call to flush the notification.

### Message bus (system integration)
Host-neutral crate `nbcad-mcp-bus` defines request/reply subjects and an
`InMemoryBus` that **CI requires**. Set `NBCAD_MCP_TRANSPORT=bus-jsonl` so an
external connector can bridge Kafka/MQTT/NATS without linking broker SDKs into
the CAD process. Details: [mcp-message-bus.md](mcp-message-bus.md).

## Proposed (not shipped here)
- In-process shared engine (today’s live mode is revisioned file co-link)
- MCP client installer (`install-mcp`) and UI launch/window control
- Full multi-window product broker (subject layout exists; UI routing next)
See [proposed-architecture.md](proposed-architecture.md).
