use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::envelope::{BusError, BusMessage};
use crate::worker::Bus;

#[derive(Default)]
struct Inner {
    /// Exact-subject queues.
    queues: HashMap<String, VecDeque<BusMessage>>,
}

/// Process-local queue that speaks the same request/reply contract as
/// NATS/Kafka/MQTT adapters. **Required** by crate integration tests.
#[derive(Clone, Default)]
pub struct InMemoryBus {
    inner: Arc<Mutex<Inner>>,
    cvar: Arc<Condvar>,
}

impl InMemoryBus {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&self, message: BusMessage) {
        let mut guard = self.inner.lock().expect("mcp-bus mutex");
        guard
            .queues
            .entry(message.subject.clone())
            .or_default()
            .push_back(message);
        self.cvar.notify_all();
    }

    fn pop_exact(&self, subject: &str) -> Option<BusMessage> {
        let mut guard = self.inner.lock().expect("mcp-bus mutex");
        guard
            .queues
            .get_mut(subject)
            .and_then(|queue| queue.pop_front())
    }

    fn wait_pop(&self, subject: &str, timeout: Duration) -> Result<BusMessage, BusError> {
        let deadline = Instant::now() + timeout;
        let mut guard = self.inner.lock().expect("mcp-bus mutex");
        loop {
            if let Some(queue) = guard.queues.get_mut(subject) {
                if let Some(message) = queue.pop_front() {
                    return Ok(message);
                }
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(BusError::Timeout);
            }
            let wait = deadline.saturating_duration_since(now);
            let (next, wait_result) = self.cvar.wait_timeout(guard, wait).expect("mcp-bus wait");
            guard = next;
            if wait_result.timed_out()
                && guard
                    .queues
                    .get(subject)
                    .map(|queue| queue.is_empty())
                    .unwrap_or(true)
            {
                return Err(BusError::Timeout);
            }
        }
    }
}

impl Bus for InMemoryBus {
    fn publish(&self, message: BusMessage) -> Result<(), BusError> {
        self.push(message);
        Ok(())
    }

    fn request(&self, message: BusMessage, timeout: Duration) -> Result<BusMessage, BusError> {
        let reply_to = message.reply_to.clone().ok_or(BusError::MissingReplyTo)?;
        let correlation_id = message.correlation_id.clone();
        self.publish(message)?;
        let started = Instant::now();
        loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(BusError::Timeout);
            }
            let reply = self.wait_pop(&reply_to, remaining)?;
            if reply.correlation_id == correlation_id {
                return Ok(reply);
            }
            // Unrelated frame on shared inbox — requeue and keep waiting.
            self.push(reply);
        }
    }

    fn recv(&self, subject: &str, timeout: Duration) -> Result<BusMessage, BusError> {
        self.wait_pop(subject, timeout)
    }

    fn try_recv(&self, subject: &str) -> Result<Option<BusMessage>, BusError> {
        Ok(self.pop_exact(subject))
    }
}
