# OKRs — mcp-server

Parent directions: [../docs/goals.md](../docs/goals.md) · As-built:
[../docs/mcp-harness.md](../docs/mcp-harness.md).

## M1 — Soft steerable surface (branch)

| KR | Target | Status |
|----|--------|--------|
| M1.1 | Default advertised tools ≪ full catalog in `dynamic` mode | On `feat/3mf-print-export` |
| M1.2 | Out-of-focus tools stay callable | On branch |
| M1.3 | 109 modeling tools tagged in disclosure matrix | This PR |
| M1.4 | Pack matrix CI (list + per-pack representatives) | On branch |

## M2 — Escape hatch (branch)

| KR | Target | Status |
|----|--------|--------|
| M2.1 | `full_static` advertises full registry | On branch |
| M2.2 | `cad_list_all_tools` always available | On branch |

## M3 — Snapshot bridge

| KR | Target | Status |
|----|--------|--------|
| M3.1 | `cad_list_sessions` / `cad_attach` / refresh / detach (read-only) | On branch |
| M3.2 | UUID ids + desktop publisher + heartbeat metadata | On branch |
| M3.3 | Revisioned MCP↔UI sync (`cad_attach mode=live` + writer lock) | Shipped — file co-link; HTTP/Tauri both directions tested |

## M4 — Print export (branch)

| KR | Target | Status |
|----|--------|--------|
| M4.1 | `solid_export_3mf` / STL / STEP + `material_catalog` | On branch |
| M4.2 | Slicer Metadata = compatible hints, not full project | By design |

## M5 — Operability

| KR | Target | Status |
|----|--------|--------|
| M5.1 | `cargo test --manifest-path mcp-server/Cargo.toml` with OCCT | Required for merge |
| M5.2 | `.github/workflows/mcp-server.yml` | Required path |

## M6 — Message-queue transport

| KR | Target | Status |
|----|--------|--------|
| M6.1 | `nbcad-mcp-bus` request/reply + `InMemoryBus` CI tests | Shipped |
| M6.2 | `NBCAD_MCP_TRANSPORT=bus-jsonl` bridge for external Kafka/MQTT/NATS connectors | Shipped |
| M6.3 | MCP tools/call round-trip test **through** the bus (OCCT) | Shipped |
| M6.4 | First-party NATS/Kafka connector binary (optional) | Next |

## M7 — MCP protocol 2026-07-28

| KR | Target | Status |
|----|--------|--------|
| M7.1 | `server/discover` + per-request `_meta` (`2026-07-28`) is the recommended path | This PR |
| M7.2 | `initialize` remains a compatibility pathway (not recommended); first reply includes the 2026 success manual | This PR |
| M7.3 | Unsupported version → JSON-RPC `-32022` | This PR |
| M7.4 | `resources/list|read` + `prompts/list|get` for product surfaces | This PR |
