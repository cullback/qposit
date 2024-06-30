use askama::Template;

use crate::{
    actors::book_service::BookData,
    models::{book::Book, market::Market},
};

use super::{book::BookHtml, format_timestamp_as_string};

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    expires_at: String,
    market: Market,
    /// Comma-separated list of book IDs TODO
    books: String,
    orderbooks: Vec<BookHtml>,
}

impl MarketPage {
    pub fn new(username: String, market: Market, books: Vec<(Book, BookData)>) -> Self {
        Self {
            username,
            expires_at: format_timestamp_as_string(market.expires_at),
            market,
            books: books
                .iter()
                .map(|(book, _)| book.id.to_string())
                .collect::<Vec<_>>()
                .join(","),
            orderbooks: books
                .into_iter()
                .map(|(book, orderbook)| BookHtml::new(book, &orderbook))
                .collect(),
        }
    }
}
