# crates/ index

Host-neutral Rust CAD crates (shared by UI / WASM / MCP where applicable).

| Crate | Role |
|-------|------|
| `core` / `sketch` / `solid` / `occt` / `export` / `wasm` | Geometry / document / history / adapters |
| `mcp-bus` | Transport-agnostic MCP request/reply bus (`InMemoryBus` required in CI; NATS/Kafka/MQTT-shaped subjects) |

MCP tool surface lives in [../mcp-server/](../mcp-server/), not here.
Bus notes: [../docs/mcp-message-bus.md](../docs/mcp-message-bus.md).
Agentic docs: [../docs/agentic/INDEX.md](../docs/agentic/INDEX.md).
