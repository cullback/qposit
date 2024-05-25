use askama::Template;
use lobster::BookId;

use crate::models::{book::Book, market::Market};

struct BookView {
    id: BookId,
    market_id: i64,
    title: String,
    last_trade_price: String,
    mid_price: String,
}

impl From<Book> for BookView {
    fn from(book: Book) -> Self {
        let mid_price = match (book.best_bid_price, book.best_ask_price) {
            (Some(bid), Some(ask)) => format!("{:.2}", (bid + ask) as f32 / 200.0),
            (Some(bid), None) => format!("{:.2}", bid as f32 / 100.0),
            (None, Some(ask)) => format!("{:.2}", ask as f32 / 100.0),
            _ => "N/A".to_string(),
        };
        Self {
            id: book.id,
            market_id: book.market_id,
            title: book.title,
            last_trade_price: format!("{:.2}", book.last_trade_price.unwrap_or(0) as f32 / 100.0),
            mid_price,
        }
    }
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomePage {
    /// Username for header. Empty string if not logged in.
    username: String,
    markets: Vec<(Market, Vec<BookView>)>,
}

impl HomePage {
    pub fn new(username: String, markets: Vec<(Market, Vec<Book>)>) -> Self {
        Self {
            username,
            markets: markets
                .into_iter()
                .map(|(market, books)| {
                    let books = books.into_iter().map(BookView::from).collect();
                    (market, books)
                })
                .collect(),
        }
    }
}
