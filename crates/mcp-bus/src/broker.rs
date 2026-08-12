//! In-process window/document registry for the future multi-window broker (#12).
//!
//! Registration and listing go through the same request/reply bus subjects as
//! modeling traffic — tests cannot bypass the queue.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::envelope::{BusError, BusHeaders, BusMessage};
use crate::subjects::{response_subject, DocumentRoute};
use crate::worker::{process_one, Bus, RpcHandler};
use crate::BUS_SCHEMA_VERSION;

/// Single control subject for broker list/register/unregister RPCs.
pub fn broker_control_subject() -> String {
    "nbcad.mcp.broker.control".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowEntry {
    pub document_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl WindowEntry {
    pub fn route(&self) -> DocumentRoute {
        DocumentRoute {
            document_id: self.document_id.clone(),
            window_id: self.window_id.clone(),
        }
    }

    fn key(&self) -> String {
        self.route().token()
    }
}

#[derive(Default)]
pub struct WindowRegistry {
    entries: Mutex<BTreeMap<String, WindowEntry>>,
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&self, entry: WindowEntry) {
        let key = entry.key();
        self.entries
            .lock()
            .expect("broker registry")
            .insert(key, entry);
    }

    pub fn remove(&self, route: &DocumentRoute) -> bool {
        self.entries
            .lock()
            .expect("broker registry")
            .remove(&route.token())
            .is_some()
    }

    pub fn list(&self) -> Vec<WindowEntry> {
        self.entries
            .lock()
            .expect("broker registry")
            .values()
            .cloned()
            .collect()
    }
}

/// Handles broker list / register / unregister JSON-RPC bodies on the bus.
pub struct BrokerHandler {
    pub registry: WindowRegistry,
}

impl BrokerHandler {
    pub fn new() -> Self {
        Self {
            registry: WindowRegistry::new(),
        }
    }
}

impl Default for BrokerHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcHandler for BrokerHandler {
    fn handle_rpc(&mut self, request_json: &[u8]) -> Result<Vec<Vec<u8>>, BusError> {
        let request: Value = serde_json::from_slice(request_json)
            .map_err(|error| BusError::InvalidJson(error.to_string()))?;
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("");
        let params = request.get("params").cloned().unwrap_or_else(|| json!({}));

        let result = match method {
            "broker/list" | "nbcad.broker.list" => json!({
                "schema": BUS_SCHEMA_VERSION,
                "windows": self.registry.list(),
            }),
            "broker/register" | "nbcad.broker.register" => {
                let entry: WindowEntry = serde_json::from_value(params)
                    .map_err(|error| BusError::Handler(error.to_string()))?;
                self.registry.upsert(entry.clone());
                json!({ "registered": true, "window": entry })
            }
            "broker/unregister" | "nbcad.broker.unregister" => {
                let document_id = params
                    .get("document_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusError::Handler("document_id required".into()))?
                    .to_string();
                let window_id = params
                    .get("window_id")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let route = DocumentRoute {
                    document_id,
                    window_id,
                };
                let removed = self.registry.remove(&route);
                json!({ "unregistered": removed, "route": route.token() })
            }
            other => {
                return Err(BusError::Handler(format!(
                    "unknown broker method: {other}"
                )));
            }
        };

        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        });
        Ok(vec![serde_json::to_vec(&response)
            .map_err(|error| BusError::Handler(error.to_string()))?])
    }
}

/// Serve one broker control request from the bus.
pub fn process_broker_one<B: Bus>(
    bus: &B,
    handler: &mut BrokerHandler,
    timeout: Duration,
) -> Result<usize, BusError> {
    process_one(bus, &broker_control_subject(), handler, timeout)
}

fn broker_request<B: Bus>(
    bus: &B,
    method: &str,
    params: Value,
    headers: BusHeaders,
    timeout: Duration,
) -> Result<Value, BusError> {
    let correlation = uuid::Uuid::new_v4().to_string();
    let reply_to = response_subject(&DocumentRoute::document("broker"), &correlation);
    let mut message = BusMessage::request(
        broker_control_subject(),
        reply_to,
        headers,
        serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        }))
        .map_err(|error| BusError::Handler(error.to_string()))?,
    );
    message.correlation_id = correlation;
    let reply = bus.request(message, timeout)?;
    reply.payload_json()
}

/// Register a window through the bus (not a direct mutex call).
pub fn register_via_bus<B: Bus>(
    bus: &B,
    entry: &WindowEntry,
    timeout: Duration,
) -> Result<Value, BusError> {
    let mut headers = BusHeaders::new().with_document(entry.document_id.clone());
    if let Some(window_id) = &entry.window_id {
        headers = headers.with_window(window_id);
    }
    broker_request(
        bus,
        "broker/register",
        serde_json::to_value(entry).map_err(|error| BusError::Handler(error.to_string()))?,
        headers,
        timeout,
    )
}

/// List windows through the bus.
pub fn list_via_bus<B: Bus>(bus: &B, timeout: Duration) -> Result<Value, BusError> {
    broker_request(bus, "broker/list", json!({}), BusHeaders::new(), timeout)
}
