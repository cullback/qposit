use askama::Template;

use crate::{
    models::{book::Book, market::Market},
    pages::markets::OrderBook,
};

#[derive(Template)]
#[template(path = "market.html")]
pub struct MarketPage {
    username: String,
    market: Market,
    books: Vec<Book>,
    orderbooks: Vec<OrderBook>,
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
            books,
            orderbooks,
        }
    }
}
