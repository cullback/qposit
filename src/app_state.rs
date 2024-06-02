use lobster::{BookEvent, Timestamp};
use sqlx::SqlitePool;
use time::{
    format_description,
    macros::{format_description, offset},
    OffsetDateTime, UtcOffset,
};
use tokio::sync::{broadcast, mpsc};

use crate::{
    actors::{book_service::BookData, matcher_request::MatcherRequest},
    pages::OrderBook,
};

pub struct AppState {
    pub pool: SqlitePool,
    /// Sending requests to matching engine.
    pub cmd_send: mpsc::Sender<MatcherRequest>,
    /// Receiving market data events.
    pub feed_receive: broadcast::Receiver<BookEvent>,
    pub book_receive: broadcast::Receiver<BookData>,
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
        feed_receive: broadcast::Receiver<BookEvent>,
        book_receive: broadcast::Receiver<BookData>,
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

/// Pretty prints a timestamp as a string.
/// e.g. November 10, 2020 12:00:00
pub fn format_as_string(timestamp: Timestamp) -> String {
    let date = OffsetDateTime::from_unix_timestamp_nanos(i128::from(timestamp * 1000)).unwrap();
    // convert to eastern time
    let date = date.to_offset(offset!(-5));

    let format = format_description!(
        "[weekday], [month repr:long] [day padding:none], [year] at [hour]:[minute]:[second]"
    );
    date.format(&format).unwrap()
}

#[cfg(test)]
mod tests {
    use super::format_as_string;

    #[test]
    fn test_timestamp_format() {
        let timestamp = 1730829600_000000;
        let formatted = format_as_string(timestamp);
        assert_eq!(formatted, "Tuesday, November 5, 2024 at 13:00:00");
    }
}
