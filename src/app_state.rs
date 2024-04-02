use exchange::{BookEvent, Timestamp};
use tokio::sync::{broadcast, mpsc};

use crate::actors::matcher_request::MatcherRequest;

pub struct AppState {
    /// Sending requests to matching engine.
    pub cmd_send: mpsc::Sender<MatcherRequest>,
    /// Receiving market data events.
    pub feed_receive: broadcast::Receiver<BookEvent>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            cmd_send: self.cmd_send.clone(),
            feed_receive: self.feed_receive.resubscribe(),
        }
    }
}

impl AppState {
    pub fn build(
        cmd_send: mpsc::Sender<MatcherRequest>,
        feed_receive: broadcast::Receiver<BookEvent>,
    ) -> Self {
        Self {
            cmd_send,
            feed_receive,
        }
    }
}

/// Returns the current time in microseconds.
pub fn current_time_micros() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as Timestamp
}
