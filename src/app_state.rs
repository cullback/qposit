use exchange::{BookEvent, Timestamp};
use sqlx::SqlitePool;
use time::{format_description, OffsetDateTime};
use tokio::sync::{broadcast, mpsc};

use crate::{actors::matcher_request::MatcherRequest, pages::OrderBook};

pub struct AppState {
    pub db: SqlitePool,
    /// Sending requests to matching engine.
    pub cmd_send: mpsc::Sender<MatcherRequest>,
    /// Receiving market data events.
    pub feed_receive: broadcast::Receiver<BookEvent>,
    pub book_receive: broadcast::Receiver<OrderBook>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            cmd_send: self.cmd_send.clone(),
            feed_receive: self.feed_receive.resubscribe(),
            book_receive: self.book_receive.resubscribe(),
        }
    }
}

impl AppState {
    pub fn new(
        db: SqlitePool,
        cmd_send: mpsc::Sender<MatcherRequest>,
        feed_receive: broadcast::Receiver<BookEvent>,
        book_receive: broadcast::Receiver<OrderBook>,
    ) -> Self {
        Self {
            db,
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

/// Pretty prints a timestamp as a string.
/// e.g. November 10, 2020 12:00:00
pub fn format_as_string(timestamp: Timestamp) -> String {
    let timestamp_seconds = timestamp / 1_000_000;
    let thing = OffsetDateTime::from_unix_timestamp(timestamp_seconds).unwrap();
    let fmt = format_description::parse("%B %d, %Y %H:%M:%S").unwrap();
    thing.format(&fmt).unwrap()
}
