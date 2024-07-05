use askama::Template;

use crate::{
    actors::book_service::MarketData,
    models::{event::Event, market::Market},
};

use super::{book::BookHtml, format_timestamp_as_string};

#[derive(Template)]
#[template(path = "event.html")]
pub struct EventPage {
    username: String,
    expires_at: String,
    event: Event,
    /// Comma-separated list of market IDs TODO
    markets: String,
    orderbooks: Vec<BookHtml>,
}

impl EventPage {
    pub fn new(username: String, event: Event, markets: Vec<(Market, MarketData)>) -> Self {
        Self {
            username,
            expires_at: format_timestamp_as_string(event.expires_at),
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
