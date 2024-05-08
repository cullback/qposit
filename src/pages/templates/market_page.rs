use askama::Template;

use crate::models::{book::Book, market::Market};

use super::{order_form::OrderForm};

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    market: Market,
    orderbooks: Vec<(Book, OrderForm)>,
}

impl MarketPage {
    pub fn new(username: String, market: Market, books: Vec<Book>) -> Self {
        Self {
            username,
            market,
            orderbooks: books
                .into_iter()
                .map(|book| {
                    let book_id = book.id;
                    (book, OrderForm::new(book_id))
                })
                .collect(),
        }
    }
}
