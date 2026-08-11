---
type: Concept
title: MCP harness
description: Local MCP stdio automation with read-only snapshots and live UI co-link.
status: stable
updated: 2026-08-11
---

# MCP harness

`mcp-server/` provides `nbcad-mcp`: a **stdio** MCP server for local
automation and testing (no required cloud).

Canonical notes: [MCP harness](../../docs/mcp-harness.md).
Proposals: [proposed architecture](../../docs/proposed-architecture.md).

## Honest today

| Fact | Meaning |
|------|---------|
| Soft focus disclosure, `listChanged: true` | Spine + active/soft packs; hidden tools stay callable |
| Headless document per MCP process | Goldens work without UI attach |
| `cad_attach mode=read_only` | Snapshot load; no writeback |
| `cad_attach mode=live` | Writer lock + `model.json` writeback; UI polls revisions |
| Browser co-link path | Vite `/__nbcad_session/*` + `npm run smoke:colink` |

## Still proposed

1. In-process shared engine (beyond revisioned file co-link)
2. Multi-window routing later if needed (not P0)

See also the [MCP playbook](../../docs/agent-mcp.md) and
[server documentation](../../mcp-server/README.md).
