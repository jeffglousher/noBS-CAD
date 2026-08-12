use std::time::Duration;

use crate::envelope::{BusError, BusMessage};
use crate::subjects::{notify_subject, DocumentRoute};

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

/// Receive one request from `request_subject`, invoke handler, publish replies.
///
/// - **First** frame → correlated `reply_to` (request/reply).
/// - **Further** frames (e.g. `notifications/tools/list_changed`) →
///   `nbcad.mcp.<route>.notify` when `document_id` is present on the request.
pub fn process_one<B: Bus, H: RpcHandler>(
    bus: &B,
    request_subject: &str,
    handler: &mut H,
    timeout: Duration,
) -> Result<usize, BusError> {
    let request = bus.recv(request_subject, timeout)?;
    let frames = handler.handle_rpc(&request.payload)?;
    let mut published = 0usize;
    let mut frames = frames.into_iter();

    if let Some(primary) = frames.next() {
        let reply = request.reply_frame(primary)?;
        bus.publish(reply)?;
        published += 1;
    } else if request.reply_to.is_some() {
        // Notification-only inbound still unblocks a waiting request() caller.
        let reply = request.reply_frame(b"{}".to_vec())?;
        bus.publish(reply)?;
        published += 1;
    }

    if let Some(document_id) = request.headers.document_id.clone() {
        let route = DocumentRoute {
            document_id,
            window_id: request.headers.window_id.clone(),
        };
        let notify = notify_subject(&route);
        for frame in frames {
            let mut message = request.reply_frame(frame)?;
            message.subject = notify.clone();
            message.reply_to = None;
            bus.publish(message)?;
            published += 1;
        }
    } else {
        // No route — drop trailing notify frames rather than pollute reply_to.
        let _ = frames.count();
    }

    Ok(published)
}
