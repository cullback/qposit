use askama::Template;
use lobster::BookId;

use crate::{
    actors::book_service::BookData,
    app_state::format_as_string,
    models::{book::Book, market::Market},
    pages::OrderBook,
};

use super::order_form::OrderForm;

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    expires_at: String,
    market: Market,
    /// Comma-separated list of book IDs
    books: String,
    orderbooks: Vec<(Book, OrderBook, OrderForm)>,
}

impl MarketPage {
    pub fn new(username: String, market: Market, books: Vec<(Book, BookData)>) -> Self {
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
                .map(|(book, orderbook)| {
                    let book_id = book.id;
                    (book, OrderBook::from(&orderbook), OrderForm::new(book_id))
                })
                .collect(),
        }
    }
}
