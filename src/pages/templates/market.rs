use askama::Template;

use crate::{
    models::{book::Book, market::Market},
    pages::markets::OrderBook,
};

#[derive(Template)]
#[template(path = "market.html")]
pub struct Component<'a> {
    username: &'a str,
    market: Market,
    books: Vec<Book>,
    orderbooks: Vec<OrderBook>,
}

pub fn build(
    username: &str,
    market: Market,
    books: Vec<Book>,
    orderbooks: Vec<OrderBook>,
) -> String {
    Component {
        username,
        market,
        books,
        orderbooks,
    }
    .render()
    .unwrap()
}
