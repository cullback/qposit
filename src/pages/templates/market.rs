use askama::Template;

use crate::{
    actors::book_service::EventData,
    models::{event::Event, market::Market},
};

use super::{book::BookHtml, format_timestamp_as_string};

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    expires_at: String,
    market: Market,
    /// Comma-separated list of event IDs TODO
    events: String,
    orderbooks: Vec<BookHtml>,
}

impl MarketPage {
    pub fn new(username: String, market: Market, events: Vec<(Event, EventData)>) -> Self {
        Self {
            username,
            expires_at: format_timestamp_as_string(market.expires_at),
            market,
            events: events
                .iter()
                .map(|(book, _)| book.id.to_string())
                .collect::<Vec<_>>()
                .join(","),
            orderbooks: events
                .into_iter()
                .map(|(book, orderbook)| BookHtml::new(book, &orderbook))
                .collect(),
        }
    }
}
