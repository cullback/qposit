use askama::Template;
use lobster::BookId;

use crate::{app_state::format_as_string, models::market::Market};

use super::order_form::OrderForm;

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    expires_at: String,
    market: Market,
    /// Comma-separated list of book IDs
    books: String,
    orderbooks: Vec<(BookId, OrderForm)>,
}

impl MarketPage {
    pub fn new(username: String, market: Market, books: Vec<BookId>) -> Self {
        Self {
            username,
            expires_at: format_as_string(market.expires_at),
            market,
            books: books
                .iter()
                .map(|book| book.to_string())
                .collect::<Vec<_>>()
                .join(","),
            orderbooks: books
                .into_iter()
                .map(|book| (book, OrderForm::new(book)))
                .collect(),
        }
    }
}
