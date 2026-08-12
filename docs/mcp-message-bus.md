# MCP message bus (Kafka / MQTT / NATS-ready)

Priority transport pattern for embedding noBS CAD MCP into an existing
message-queued system. **Stdio remains** the Cursor/VS Code path; the bus is
the system-integration path.

Crate: [`crates/mcp-bus`](../crates/mcp-bus) (`nbcad-mcp-bus`).
Upstream architecture context: [#12 multi-window broker](https://github.com/jackControls/noBS-CAD/issues/12),
epic [#9](https://github.com/jackControls/noBS-CAD/issues/9).

## Required pattern

1. Client publishes a **request** envelope to `nbcad.mcp.<document_id>[.<window_id>].req`
2. Envelope carries `correlation_id`, `reply_to`, JSON-RPC `payload`, and routing headers
3. Worker handles JSON-RPC and publishes the **primary** reply to `reply_to`
4. Side-channel MCP notifications go to `nbcad.mcp.<route>.notify`
5. Multi-window registry RPCs use `nbcad.mcp.broker.control` (`broker/list|register|unregister`)

CI **requires** this pattern via `InMemoryBus` tests (`cargo test -p nbcad-mcp-bus`).
Kernel goldens may still call tools in-process; transport/broker tests must not.

## Broker mapping

| Broker | Mapping |
|--------|---------|
| **NATS** | Subjects as documented (best fit for request/reply + queue groups) |
| **MQTT** | Topic = subject string; QoS 1; app-level reply topic in `reply_to` |
| **Kafka** | Topic per subject family (or `.`→`_`); `correlation_id` / `reply_to` in headers; compact reply topics or inbox partitions |

Schema stamp: `nbcad.mcp-bus.v1` (`BUS_SCHEMA_VERSION`).
Correlation and session IDs are **BLAKE3 UUID v8** (internal layout version 1 in
byte 0). See `crates/id`.

## Runtime bridge (no broker SDK in CAD)

```sh
NBCAD_MCP_TRANSPORT=bus-jsonl ./nbcad-mcp
```

Each stdin line is one `BusMessage` JSON object; stdout lines are reply/notify
`BusMessage` frames. A small connector process (NATS/Kafka/MQTT client) owns
broker credentials and translates:

`broker message → BusMessage JSONL → nbcad-mcp → BusMessage JSONL → broker`

Default transport remains `stdio` when `NBCAD_MCP_TRANSPORT` is unset.

## Honest limits

- **Transport, not a scale-out kernel.** `nbcad-mcp` still owns **one in-process document**. The bus lets you *address* that worker with `document_id` / `window_id` and run it behind Kafka/MQTT/NATS. Horizontal scale means **one worker process per document**, not one shared stateless replica.
- **MCP protocol remains `2025-06-18`** (initialize handshake on stdio). The bus envelope is request/reply; it is not the `2026-07-28` spec.
- **Broker registry** (`broker/list|register|unregister`) is the control-plane contract for #12. The desktop UI does not register windows yet.

## Local proof without a broker

```sh
cargo test -p nbcad-mcp-bus
# Windows / OCCT:
cargo test --manifest-path mcp-server/Cargo.toml mcp_tools_call_must_roundtrip_through_message_bus
# Optional runtime demo (needs a built nbcad-mcp):
NBCAD_MCP_BIN=./mcp-server/target/release/nbcad-mcp node scripts/mcp-bus-jsonl-demo.mjs
```

## Non-goals (this slice)

- Embedding `rdkafka` / `paho-mqtt` / `async-nats` inside `nbcad-mcp`
- Replacing MCP protocol `2025-06-18` with the `2026-07-28` stateless core
- Full multi-window product broker UI (control-plane registry + subjects are ready; UI attach comes next)
