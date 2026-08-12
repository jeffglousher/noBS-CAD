//! Transport-agnostic MCP message bus for noBS CAD.
//!
//! # Why
//! Stdio remains the local Cursor/VS Code path. Systems that already use
//! Kafka, MQTT, or NATS need the same JSON-RPC MCP payloads delivered over a
//! **request/reply queue pattern** with explicit routing keys
//! (`document_id` / `window_id`) — the foundation for multi-window broker
//! work (upstream #12) without embedding broker SDKs in every test.
//!
//! # Contract
//! - Every request carries a `correlation_id` and optional `reply_to`.
//! - Workers publish one or more reply frames to `reply_to` (or the derived
//!   response subject) with the same `correlation_id`.
//! - Subjects are broker-neutral strings (NATS-style tokens; map 1:1 to
//!   Kafka topics or MQTT topics by replacing `.` if a broker requires it).
//! - CI **must** exercise tools through [`InMemoryBus`] (see crate tests).
//!   Direct in-process `call_tool` remains allowed for kernel goldens, but
//!   transport/broker tests cannot bypass the bus.

mod broker;
mod envelope;
mod memory;
mod subjects;
mod worker;

pub use broker::{
    broker_control_subject, list_via_bus, process_broker_one, register_via_bus, BrokerHandler,
    WindowEntry, WindowRegistry,
};
pub use envelope::{BusError, BusHeaders, BusMessage};
pub use memory::InMemoryBus;
pub use subjects::{
    broker_list_subject, notify_subject, request_subject, response_subject, DocumentRoute,
};
pub use worker::{process_one, Bus, RpcHandler};

/// Crate-level protocol tag stamped on bus headers (not the MCP protocolVersion).
pub const BUS_SCHEMA_VERSION: &str = "nbcad.mcp-bus.v1";
