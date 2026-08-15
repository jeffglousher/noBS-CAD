---
type: Concept
title: MCP harness
description: Local MCP stdio automation; recommended protocol 2026-07-28.
status: stable
updated: 2026-08-15
---

# MCP harness

`mcp-server/` provides `nbcad-mcp`: a **stdio** MCP server for local
automation and testing (no required cloud). **Recommended protocol is
`2026-07-28`** (`server/discover` + per-request `_meta`). `initialize`
(`2025-06-18`) remains a compatibility pathway only. Optional `bus-jsonl`
transport exposes the same JSON-RPC over a request/reply envelope for
Kafka/MQTT/NATS connectors.

Canonical notes: [MCP harness](../../docs/mcp-harness.md).
Message bus: [MCP message bus](../../docs/mcp-message-bus.md).
Proposals: [proposed architecture](../../docs/proposed-architecture.md).

## Honest today

| Fact | Meaning |
|------|---------|
| Soft focus disclosure, `listChanged: true` | Spine + active/soft packs; hidden tools stay callable |
| Headless document per MCP process | Goldens work without UI attach |
| `cad_attach mode=read_only` | Snapshot load; no writeback |
| `cad_attach mode=live` | Takes lock from UI-published session; writeback + UI poll |
| Browser co-link path | Vite `/__nbcad_session/*` + `npm run smoke:colink` |
| Session / correlation IDs | BLAKE3 UUID v8 (nbcad layout 1); legacy v4 dirs still attach |
| `nbcad-mcp-bus` + `InMemoryBus` | CI-required request/reply pattern; `NBCAD_MCP_TRANSPORT=bus-jsonl` bridge |
| Recommended MCP | `server/discover` + `_meta` (`2026-07-28`); success manual in first legacy reply |
| Resources / prompts | Main-surface `nbcad://` reads (document, scene, sketch, profiles, features, drawing, workspace, …) and recipes (box, solid, printable tool, STEP, drawing export, undo, invoke) |
| `initialize` pathway | Compatibility only — not recommended; handshake stays `2025-06-18` |

## Still proposed

1. In-process shared engine (beyond revisioned file co-link)
2. Full multi-window product broker (subject layout ready)
3. First-party NATS/Kafka connector binary

See also the [MCP 2026 success manual](../../docs/agentic/MCP_2026.md),
the [MCP playbook](../../docs/agent-mcp.md), and
[server documentation](../../mcp-server/README.md).
