//! Transport tests **must** go through [`nbcad_mcp_bus::InMemoryBus`].
//! These tests fail the build if the request/reply queue pattern regresses.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use nbcad_mcp_bus::{
    process_one, request_subject, response_subject, Bus, BusError, BusHeaders, BusMessage,
    DocumentRoute, InMemoryBus, RpcHandler, BUS_SCHEMA_VERSION,
};
use serde_json::{json, Value};

struct EchoHandler;

impl RpcHandler for EchoHandler {
    fn handle_rpc(&mut self, request_json: &[u8]) -> Result<Vec<Vec<u8>>, BusError> {
        let request: Value = serde_json::from_slice(request_json)
            .map_err(|error| BusError::InvalidJson(error.to_string()))?;
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let primary = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "echo": method, "bus": BUS_SCHEMA_VERSION }
        });
        let notify = json!({
            "jsonrpc": "2.0",
            "method": "notifications/tools/list_changed"
        });
        Ok(vec![
            serde_json::to_vec(&primary).unwrap(),
            serde_json::to_vec(&notify).unwrap(),
        ])
    }
}

struct CountingHandler {
    calls: Arc<Mutex<usize>>,
}

impl RpcHandler for CountingHandler {
    fn handle_rpc(&mut self, request_json: &[u8]) -> Result<Vec<Vec<u8>>, BusError> {
        *self.calls.lock().unwrap() += 1;
        let request: Value = serde_json::from_slice(request_json)
            .map_err(|error| BusError::InvalidJson(error.to_string()))?;
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let body = json!({ "jsonrpc": "2.0", "id": id, "result": { "ok": true } });
        Ok(vec![serde_json::to_vec(&body).unwrap()])
    }
}

#[test]
fn request_reply_requires_in_memory_bus_pattern() {
    let bus = InMemoryBus::new();
    let route = DocumentRoute::document("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    let req_subject = request_subject(&route);

    let worker_bus = bus.clone();
    let worker_subject = req_subject.clone();
    let worker = thread::spawn(move || {
        let mut handler = EchoHandler;
        process_one(
            &worker_bus,
            &worker_subject,
            &mut handler,
            Duration::from_secs(2),
        )
        .expect("worker should process one request");
    });

    let payload = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/list",
        "params": {}
    }))
    .unwrap();
    let correlation = "corr-pattern-1";
    let reply_to = response_subject(&route, correlation);
    let mut message = BusMessage::request(
        req_subject,
        reply_to.clone(),
        BusHeaders::new()
            .with_document(route.document_id.clone())
            .with_protocol("2025-06-18"),
        payload,
    );
    message.correlation_id = correlation.to_string();

    let reply = bus
        .request(message, Duration::from_secs(2))
        .expect("client must receive correlated reply via bus");
    assert_eq!(reply.correlation_id, correlation);
    assert_eq!(reply.subject, reply_to);
    assert_eq!(reply.headers.schema, BUS_SCHEMA_VERSION);

    let body: Value = reply.payload_json().unwrap();
    assert_eq!(body["result"]["echo"], "tools/list");
    assert_eq!(body["result"]["bus"], BUS_SCHEMA_VERSION);

    // Side-channel notification must not be the request() reply.
    let notify = bus
        .recv(
            &nbcad_mcp_bus::notify_subject(&route),
            Duration::from_secs(1),
        )
        .expect("list_changed should land on notify subject");
    let notify_body: Value = notify.payload_json().unwrap();
    assert_eq!(notify_body["method"], "notifications/tools/list_changed");

    worker.join().unwrap();
}

#[test]
fn two_document_routes_do_not_cross_talk() {
    let bus = InMemoryBus::new();
    let route_a = DocumentRoute::document("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb");
    let route_b = DocumentRoute::document("cccccccc-cccc-4ccc-8ccc-cccccccccccc");

    let calls_a = Arc::new(Mutex::new(0usize));
    let calls_b = Arc::new(Mutex::new(0usize));

    let bus_a = bus.clone();
    let subject_a = request_subject(&route_a);
    let counter_a = Arc::clone(&calls_a);
    let worker_a = thread::spawn(move || {
        let mut handler = CountingHandler { calls: counter_a };
        process_one(&bus_a, &subject_a, &mut handler, Duration::from_secs(2)).unwrap();
    });

    let bus_b = bus.clone();
    let subject_b = request_subject(&route_b);
    let counter_b = Arc::clone(&calls_b);
    let worker_b = thread::spawn(move || {
        let mut handler = CountingHandler { calls: counter_b };
        process_one(&bus_b, &subject_b, &mut handler, Duration::from_secs(2)).unwrap();
    });

    for (route, id) in [(&route_a, 1), (&route_b, 2)] {
        let correlation = format!("corr-{id}");
        let message = {
            let mut message = BusMessage::request(
                request_subject(route),
                response_subject(route, &correlation),
                BusHeaders::new().with_document(route.document_id.clone()),
                serde_json::to_vec(&json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "ping",
                    "params": {}
                }))
                .unwrap(),
            );
            message.correlation_id = correlation;
            message
        };
        let reply = bus.request(message, Duration::from_secs(2)).unwrap();
        let body: Value = reply.payload_json().unwrap();
        assert_eq!(body["id"], id);
    }

    worker_a.join().unwrap();
    worker_b.join().unwrap();
    assert_eq!(*calls_a.lock().unwrap(), 1);
    assert_eq!(*calls_b.lock().unwrap(), 1);
}

#[test]
fn missing_reply_to_is_a_hard_bus_error() {
    let bus = InMemoryBus::new();
    let message = BusMessage {
        subject: "nbcad.mcp.x.req".into(),
        correlation_id: "c".into(),
        reply_to: None,
        headers: BusHeaders::new(),
        payload: b"{}".to_vec(),
    };
    let err = bus
        .request(message, Duration::from_millis(50))
        .expect_err("request/reply pattern requires reply_to");
    assert_eq!(err, BusError::MissingReplyTo);
}
