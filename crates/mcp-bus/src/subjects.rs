//! Broker-neutral subject / topic layout.
//!
//! ## Mapping
//! | Broker | How to map |
//! |--------|------------|
//! | **NATS** | Use subjects as-is (`nbcad.mcp.<doc>.req`) |
//! | **MQTT** | Same string as topic (QoS 1 for req/reply apps) |
//! | **Kafka** | Topic = subject with `.` → `_` *or* keep dotted names if allowed; put `correlation_id` / `reply_to` in headers |
//!
//! Request/reply is the required pattern: clients never assume sticky
//! connections. Workers may be scaled behind a queue group later.

/// Route keys that select which MCP document/window consumes a request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentRoute {
    pub document_id: String,
    pub window_id: Option<String>,
}

impl DocumentRoute {
    pub fn document(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            window_id: None,
        }
    }

    pub fn with_window(mut self, window_id: impl Into<String>) -> Self {
        self.window_id = Some(window_id.into());
        self
    }

    /// Token used inside subjects. Window is optional suffix.
    pub fn token(&self) -> String {
        match &self.window_id {
            Some(window) => format!("{}.{}", self.document_id, window),
            None => self.document_id.clone(),
        }
    }
}

pub fn request_subject(route: &DocumentRoute) -> String {
    format!("nbcad.mcp.{}.req", route.token())
}

pub fn response_subject(route: &DocumentRoute, correlation_id: &str) -> String {
    format!("nbcad.mcp.{}.res.{}", route.token(), correlation_id)
}

pub fn notify_subject(route: &DocumentRoute) -> String {
    format!("nbcad.mcp.{}.notify", route.token())
}

pub fn broker_list_subject() -> String {
    "nbcad.mcp.broker.list".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subjects_are_stable_for_broker_mapping() {
        let route = DocumentRoute::document("11111111-1111-4111-8111-111111111111")
            .with_window("win-a");
        assert_eq!(
            request_subject(&route),
            "nbcad.mcp.11111111-1111-4111-8111-111111111111.win-a.req"
        );
        assert_eq!(
            response_subject(&route, "corr-1"),
            "nbcad.mcp.11111111-1111-4111-8111-111111111111.win-a.res.corr-1"
        );
        assert_eq!(broker_list_subject(), "nbcad.mcp.broker.list");
    }
}
