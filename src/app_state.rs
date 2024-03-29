use exchange::Timestamp;
use tokio::sync::mpsc;

use crate::actors::matcher_request::MatcherRequest;

#[derive(Clone)]
pub struct AppState {
    /// Sending requests to matching engine.
    pub matcher: mpsc::Sender<MatcherRequest>,
}

/// Returns the current time in microseconds.
pub fn current_time_micros() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as Timestamp
}

impl AppState {
    pub async fn build(matcher: mpsc::Sender<MatcherRequest>) -> Self {
        Self { matcher }
    }
}
