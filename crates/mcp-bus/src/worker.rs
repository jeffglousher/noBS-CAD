use std::time::Duration;

use serde_json::{json, Value};

use crate::envelope::{BusError, BusMessage};
use crate::subjects::notify_subject;

/// Minimal bus surface shared by in-memory and external brokers.
pub trait Bus: Send + Sync {
    fn publish(&self, message: BusMessage) -> Result<(), BusError>;
    fn request(&self, message: BusMessage, timeout: Duration) -> Result<BusMessage, BusError>;
    fn recv(&self, subject: &str, timeout: Duration) -> Result<BusMessage, BusError>;
    fn try_recv(&self, subject: &str) -> Result<Option<BusMessage>, BusError>;
}

/// JSON-RPC handler bound to one document worker.
pub trait RpcHandler {
    /// Handle one JSON-RPC request body; return zero or more JSON-RPC frames
    /// (primary result first, then optional `notifications/*`).
    fn handle_rpc(&mut self, request_json: &[u8]) -> Result<Vec<Vec<u8>>, BusError>;
}

/// Turn handler frames into bus messages: first frame → `reply_to`, rest → notify.
pub fn complete_request(
    request: &BusMessage,
    frames: Vec<Vec<u8>>,
) -> Result<Vec<BusMessage>, BusError> {
    let mut outgoing = Vec::with_capacity(frames.len().max(1));
    let mut frames = frames.into_iter();
    match frames.next() {
        Some(primary) => outgoing.push(request.reply_frame(primary)?),
        None if request.reply_to.is_some() => {
            outgoing.push(request.reply_frame(b"{}".to_vec())?);
        }
        None => {}
    }
    if let Some(route) = request.headers.route() {
        let notify = notify_subject(&route);
        for frame in frames {
            outgoing.push(request.notify_frame(notify.clone(), frame));
        }
    }
    Ok(outgoing)
}

/// Correlated JSON-RPC error on `reply_to` so clients do not hang.
pub fn jsonrpc_error_frames(request: &BusMessage, code: i64, message: &str) -> Vec<BusMessage> {
    let id = request
        .payload_json()
        .ok()
        .and_then(|value| value.get("id").cloned())
        .unwrap_or(Value::Null);
    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    });
    let payload = serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec());
    request.reply_frame(payload).into_iter().collect()
}

/// Receive one request, invoke handler, publish replies (or a JSON-RPC error).
pub fn process_one<B: Bus, H: RpcHandler>(
    bus: &B,
    request_subject: &str,
    handler: &mut H,
    timeout: Duration,
) -> Result<usize, BusError> {
    let request = bus.recv(request_subject, timeout)?;
    let outgoing = match handler.handle_rpc(&request.payload) {
        Ok(frames) => complete_request(&request, frames)?,
        Err(error) => jsonrpc_error_frames(&request, -32603, &error.to_string()),
    };
    let published = outgoing.len();
    for message in outgoing {
        bus.publish(message)?;
    }
    Ok(published)
}
