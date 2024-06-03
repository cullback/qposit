use askama::Template;
use lobster::BookId;

use crate::models::book::Book;

use super::{
    format_price_to_string,
    order_form::OrderForm,
    orderbook::{do_side, PriceLevel},
};

#[derive(Template, Debug, Clone)]
#[template(path = "book.html")]
pub struct BookHtml {
    pub book_id: BookId,
    pub title: String,
    pub last_price: String,
    pub mid_price: String,
    pub volume: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub order_form: OrderForm,
}

impl BookHtml {
    pub fn new(book: Book, orderbook: lobster::OrderBook) -> Self {
        let last_price = format_price_to_string(book.last_trade_price.unwrap_or(0));
        let volume = format!("{:.2}", book.volume as f32 / 10000.0);
        let mid_price = match (book.best_bid_price, book.best_ask_price) {
            (Some(bid), Some(ask)) => format_price_to_string((bid + ask) / 2),
            (Some(bid), None) => format_price_to_string(bid),
            (None, Some(ask)) => format_price_to_string(ask),
            _ => "N/A".to_string(),
        };
        Self {
            book_id: book.id,
            title: book.title,
            last_price,
            mid_price,
            volume,
            bids: do_side(orderbook.bids()),
            asks: do_side(orderbook.asks()),
            order_form: OrderForm::new(book.id),
        }
    }
}
