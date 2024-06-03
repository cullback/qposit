use askama::Template;

use crate::{
    app_state::format_as_string,
    models::{book::Book, market::Market},
};

use super::book::BookHtml;

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    expires_at: String,
    market: Market,
    /// Comma-separated list of book IDs
    books: String,
    orderbooks: Vec<BookHtml>,
}

impl MarketPage {
    pub fn new(username: String, market: Market, books: Vec<(Book, lobster::OrderBook)>) -> Self {
        Self {
            username,
            expires_at: format_as_string(market.expires_at),
            market,
            books: books
                .iter()
                .map(|(book, _)| book.id.to_string())
                .collect::<Vec<_>>()
                .join(","),
            orderbooks: books
                .into_iter()
                .map(|(book, orderbook)| BookHtml::new(book, orderbook))
                .collect(),
        }
    }
}
