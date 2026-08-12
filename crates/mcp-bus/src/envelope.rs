use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::subjects::DocumentRoute;
use crate::BUS_SCHEMA_VERSION;

mod payload_bytes {
    use serde::de::{self, SeqAccess, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        let text = std::str::from_utf8(bytes).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(text)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        struct PayloadVisitor;
        impl<'de> Visitor<'de> for PayloadVisitor {
            type Value = Vec<u8>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("UTF-8 JSON string or byte array")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Vec<u8>, E> {
                Ok(value.as_bytes().to_vec())
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Vec<u8>, E> {
                Ok(value.into_bytes())
            }
            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Vec<u8>, E> {
                Ok(value.to_vec())
            }
            fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Vec<u8>, E> {
                Ok(value)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<u8>, A::Error> {
                let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(byte) = seq.next_element::<u8>()? {
                    bytes.push(byte);
                }
                Ok(bytes)
            }
        }
        deserializer.deserialize_any(PayloadVisitor)
    }
}

/// Stable routing / tracing headers carried beside the JSON-RPC body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusHeaders {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    #[serde(default = "default_schema")]
    pub schema: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

fn default_schema() -> String {
    BUS_SCHEMA_VERSION.to_string()
}

impl Default for BusHeaders {
    fn default() -> Self {
        Self {
            document_id: None,
            window_id: None,
            protocol_version: None,
            schema: default_schema(),
            extra: BTreeMap::new(),
        }
    }
}

impl BusHeaders {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_document(mut self, document_id: impl Into<String>) -> Self {
        self.document_id = Some(document_id.into());
        self
    }

    pub fn with_window(mut self, window_id: impl Into<String>) -> Self {
        let window_id = window_id.into();
        self.window_id = if window_id.is_empty() {
            None
        } else {
            Some(window_id)
        };
        self
    }

    pub fn with_protocol(mut self, protocol_version: impl Into<String>) -> Self {
        self.protocol_version = Some(protocol_version.into());
        self
    }

    /// Route used for request/notify subjects. Empty window ids are ignored.
    pub fn route(&self) -> Option<DocumentRoute> {
        let document_id = self.document_id.as_deref()?.trim();
        if document_id.is_empty() {
            return None;
        }
        let window_id = self
            .window_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        Some(DocumentRoute {
            document_id: document_id.to_string(),
            window_id,
        })
    }
}

/// One frame on the bus: subject + correlation + JSON-RPC payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusMessage {
    pub subject: String,
    pub correlation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(default)]
    pub headers: BusHeaders,
    /// JSON-RPC body. Serialized as a UTF-8 string (legacy byte arrays still parse).
    #[serde(with = "payload_bytes")]
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
            correlation_id: nbcad_id::mint_string(nbcad_id::Domain::Correlation),
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

    pub fn notify_frame(&self, subject: impl Into<String>, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            subject: subject.into(),
            correlation_id: self.correlation_id.clone(),
            reply_to: None,
            headers: self.headers.clone(),
            payload: payload.into(),
        }
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
