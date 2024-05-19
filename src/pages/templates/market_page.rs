use askama::Template;
use exchange::BookId;

use crate::models::market::Market;

use super::order_form::OrderForm;

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    market: Market,
    /// Comma-separated list of book IDs
    books: String,
    orderbooks: Vec<(BookId, OrderForm)>,
}

impl MarketPage {
    pub fn new(username: String, market: Market, books: Vec<BookId>) -> Self {
        Self {
            username,
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
