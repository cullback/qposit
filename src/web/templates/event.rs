use askama::Template;

use crate::{
    models::{event::Event, market::Market},
    services::book_service::MarketData,
};

use super::{format_timestamp_as_string, market::BookHtml};

#[derive(Template)]
#[template(path = "event.html")]
pub struct EventPage {
    username: String,
    event_time: String,
    event: Event,
    /// Comma-separated list of market IDs TODO
    markets: String,
    orderbooks: Vec<BookHtml>,
}

impl EventPage {
    pub fn new(username: String, event: Event, markets: Vec<(Market, MarketData)>) -> Self {
        Self {
            username,
            event_time: format_timestamp_as_string(event.event_time),
            event,
            markets: markets
                .iter()
                .map(|(book, _)| book.id.to_string())
                .collect::<Vec<_>>()
                .join(","),
            orderbooks: markets
                .into_iter()
                .map(|(book, orderbook)| BookHtml::new(book, &orderbook))
                .collect(),
        }
    }
}
