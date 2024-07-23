use lobster::{MarketUpdate, Timestamp};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};

use crate::services::{book_service::MarketData, matcher_request::MatcherRequest};

pub struct AppState {
    pub pool: SqlitePool,
    /// Sending requests to matching engine.
    pub cmd_send: mpsc::Sender<MatcherRequest>,
    /// Receiving event data markets.
    pub feed_receive: broadcast::Receiver<MarketUpdate>,
    pub book_receive: broadcast::Receiver<MarketData>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            cmd_send: self.cmd_send.clone(),
            feed_receive: self.feed_receive.resubscribe(),
            book_receive: self.book_receive.resubscribe(),
        }
    }
}

impl AppState {
    pub fn new(
        pool: SqlitePool,
        cmd_send: mpsc::Sender<MatcherRequest>,
        feed_receive: broadcast::Receiver<MarketUpdate>,
        book_receive: broadcast::Receiver<MarketData>,
    ) -> Self {
        Self {
            pool,
            cmd_send,
            feed_receive,
            book_receive,
        }
    }
}

/// Returns the current time in microseconds.
#[allow(clippy::cast_possible_truncation)]
pub fn current_time_micros() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as Timestamp
}
