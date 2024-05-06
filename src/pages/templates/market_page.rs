use askama::Template;

use crate::models::{book::Book, market::Market};

use super::{order_form::OrderForm, orderbook::OrderBook};

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    market: Market,
    orderbooks: Vec<(Book, OrderBook, OrderForm)>,
}

impl MarketPage {
    pub fn new(
        username: String,
        market: Market,
        books: Vec<Book>,
        orderbooks: Vec<OrderBook>,
    ) -> Self {
        Self {
            username,
            market,
            orderbooks: books
                .into_iter()
                .zip(orderbooks.into_iter())
                .map(|(book, orderbook)| {
                    let book_id = book.id;
                    (book, orderbook, OrderForm::new(book_id))
                })
                .collect(),
        }
    }
}
