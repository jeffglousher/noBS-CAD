use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::envelope::{BusError, BusMessage};
use crate::worker::Bus;

#[derive(Default)]
struct Inner {
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

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn push(&self, message: BusMessage) {
        let mut guard = self.lock();
        guard
            .queues
            .entry(message.subject.clone())
            .or_default()
            .push_back(message);
        self.cvar.notify_all();
    }

    fn pop_exact(&self, subject: &str) -> Option<BusMessage> {
        self.lock()
            .queues
            .get_mut(subject)
            .and_then(|queue| queue.pop_front())
    }

    fn wait_pop(&self, subject: &str, timeout: Duration) -> Result<BusMessage, BusError> {
        self.wait_pop_if(subject, timeout, |_| true)
    }

    fn wait_pop_matching(
        &self,
        subject: &str,
        correlation_id: &str,
        timeout: Duration,
    ) -> Result<BusMessage, BusError> {
        self.wait_pop_if(subject, timeout, |message| {
            message.correlation_id == correlation_id
        })
    }

    fn wait_pop_if(
        &self,
        subject: &str,
        timeout: Duration,
        matches: impl Fn(&BusMessage) -> bool,
    ) -> Result<BusMessage, BusError> {
        let deadline = Instant::now() + timeout;
        let mut guard = self.lock();
        loop {
            if let Some(queue) = guard.queues.get_mut(subject) {
                if let Some(index) = queue.iter().position(|message| matches(message)) {
                    return Ok(queue.remove(index).expect("index from position"));
                }
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(BusError::Timeout);
            }
            let wait = deadline.saturating_duration_since(now);
            let (next, wait_result) = match self.cvar.wait_timeout(guard, wait) {
                Ok(pair) => pair,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard = next;
            if wait_result.timed_out() {
                if let Some(queue) = guard.queues.get_mut(subject) {
                    if let Some(index) = queue.iter().position(|message| matches(message)) {
                        return Ok(queue.remove(index).expect("index from position"));
                    }
                }
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
        self.wait_pop_matching(&reply_to, &correlation_id, timeout)
    }

    fn recv(&self, subject: &str, timeout: Duration) -> Result<BusMessage, BusError> {
        self.wait_pop(subject, timeout)
    }

    fn try_recv(&self, subject: &str) -> Result<Option<BusMessage>, BusError> {
        Ok(self.pop_exact(subject))
    }
}
