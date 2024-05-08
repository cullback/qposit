use exchange::{BookEvent, Timestamp};
use kanal::{AsyncReceiver, AsyncSender};
use sqlx::SqlitePool;

use crate::{actors::matcher_request::MatcherRequest, pages::OrderBook};

pub struct AppState {
    pub db: SqlitePool,
    /// Sending requests to matching engine.
    pub cmd_send: AsyncSender<MatcherRequest>,
    /// Receiving market data events.
    pub feed_receive: AsyncReceiver<BookEvent>,
    pub book_receive: AsyncReceiver<OrderBook>,
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
        cmd_send: AsyncSender<MatcherRequest>,
        feed_receive: AsyncReceiver<BookEvent>,
        book_receive: AsyncReceiver<OrderBook>,
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
