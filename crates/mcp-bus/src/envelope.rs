use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::BUS_SCHEMA_VERSION;

/// Stable routing / tracing headers carried beside the JSON-RPC body.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusHeaders {
    /// MCP document / session target (UUID or headless token).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    /// UI window target when multi-window broker is active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_id: Option<String>,
    /// MCP protocol version the client intends (e.g. `2025-06-18`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    /// Bus envelope schema (`nbcad.mcp-bus.v1`).
    #[serde(default = "default_schema")]
    pub schema: String,
    /// Extra broker-specific keys (kept ordered for golden tests).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

fn default_schema() -> String {
    BUS_SCHEMA_VERSION.to_string()
}

impl BusHeaders {
    pub fn new() -> Self {
        Self {
            schema: default_schema(),
            ..Self::default()
        }
    }

    pub fn with_document(mut self, document_id: impl Into<String>) -> Self {
        self.document_id = Some(document_id.into());
        self
    }

    pub fn with_window(mut self, window_id: impl Into<String>) -> Self {
        self.window_id = Some(window_id.into());
        self
    }

    pub fn with_protocol(mut self, protocol_version: impl Into<String>) -> Self {
        self.protocol_version = Some(protocol_version.into());
        self
    }
}

/// One frame on the bus: subject + correlation + JSON-RPC payload bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusMessage {
    pub subject: String,
    pub correlation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(default)]
    pub headers: BusHeaders,
    /// UTF-8 JSON-RPC request or response body.
    pub payload: Vec<u8>,
}

impl BusMessage {
    pub fn request(
        subject: impl Into<String>,
        reply_to: impl Into<String>,
        headers: BusHeaders,
        payload: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            subject: subject.into(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            reply_to: Some(reply_to.into()),
            headers,
            payload: payload.into(),
        }
    }

    pub fn reply_frame(&self, payload: impl Into<Vec<u8>>) -> Result<Self, BusError> {
        let reply_to = self.reply_to.clone().ok_or(BusError::MissingReplyTo)?;
        Ok(Self {
            subject: reply_to,
            correlation_id: self.correlation_id.clone(),
            reply_to: None,
            headers: self.headers.clone(),
            payload: payload.into(),
        })
    }

    pub fn payload_json(&self) -> Result<serde_json::Value, BusError> {
        serde_json::from_slice(&self.payload)
            .map_err(|error| BusError::InvalidJson(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusError {
    MissingReplyTo,
    Timeout,
    Disconnected,
    InvalidJson(String),
    Handler(String),
    NoSubscriber(String),
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingReplyTo => write!(f, "bus request is missing reply_to"),
            Self::Timeout => write!(f, "bus request timed out"),
            Self::Disconnected => write!(f, "bus disconnected"),
            Self::InvalidJson(message) => write!(f, "invalid JSON payload: {message}"),
            Self::Handler(message) => write!(f, "handler error: {message}"),
            Self::NoSubscriber(subject) => write!(f, "no subscriber for subject '{subject}'"),
        }
    }
}

impl std::error::Error for BusError {}
