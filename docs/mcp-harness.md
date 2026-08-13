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
| Protocol | **Recommended `2026-07-28`:** `server/discover` + per-request `_meta`. `initialize` (`2025-06-18`) is a compatibility pathway only — first reply includes the runtime-upgrade manual. Unsupported versions return JSON-RPC `-32022`. |
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

- **Desktop (Tauri):** `mcp_session_bridge_reserve|write|heartbeat|poll` (live writer.json + MCP-preserving heartbeat).
- **Browser/WASM (Vite):** `/__nbcad_session/*` middleware (`scripts/session-http-bridge.mjs`).

Attach modes:
1. `cad_list_sessions` — UUID dirs; heartbeat `age_ms` / `stale`; `writer` lock metadata.
2. `cad_attach` with `mode: "read_only"` (default) — load snapshot; **no** writeback; does not claim `writer.json`.
3. `cad_attach` with `mode: "live"` — requires a **fresh** heartbeat; **takes** the writer lock from `ui` or `none` (`writer=mcp`). After mutating tools, MCP writebacks `model.json` and bumps generation (`source: "mcp"`). UI polls and applies. While MCP holds the lock, UI publish/heartbeat must not clobber that revision.
4. Writer conflict: if `writer.json` is `ui` *after* live attach, MCP mutates fail until `cad_refresh` (loads the UI model) / the lock returns to MCP. `cad_refresh` does not steal the lock.
5. `cad_refresh` / `cad_detach` — re-read or clear attach; live detach releases the writer lock. `cad_attach` to another session detaches first (so `writer=mcp` is not stranded); a failed switch keeps the current attach.

Tests: `npm run test:session-bridge` (HTTP both directions, no WASM). Smoke: `npm run smoke:colink` (browser publish → MCP revision → heartbeat must preserve MCP → UI apply).
Installer / UI launch: [#32](https://github.com/jackControls/noBS-CAD/pull/32).
Build and tool flow: [mcp-server/README.md](../mcp-server/README.md).
Day-to-day playbook: [agent-mcp.md](agent-mcp.md).

### Stdio (current supported path)
Agents and CI spawn `nbcad-mcp` as an MCP stdio server. One process owns one
document. Prefer `solid_export_3mf` for slicer handoff; STEP for CAD interchange.

**Recommended:** probe with `server/discover`, then put protocol version in
`params._meta`. Manual: [agentic/MCP_2026.md](agentic/MCP_2026.md).

`initialize` remains a compatibility pathway so older clients keep working. It
is not recommended. Do not treat `initialize` returning `2026-07-28` as
success — the spec retired that handshake. The first `initialize` result
(and the first legacy `tools/call` if there was no handshake) includes the
runtime-upgrade recipe; the call still succeeds.

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
