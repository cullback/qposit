use askama::Template;
use lobster::BookId;

use crate::models::{book::Book, market::Market};

struct BookView {
    pub id: BookId,
    pub market_id: i64,
    pub title: String,
    pub last_trade_price: String,
}

impl From<Book> for BookView {
    fn from(book: Book) -> Self {
        Self {
            id: book.id,
            market_id: book.market_id,
            title: book.title,
            last_trade_price: format!("{:.2}", book.last_trade_price.unwrap_or(0) as f32 / 100.0),
        }
    }
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomePage {
    pub username: String,
    pub markets: Vec<(Market, Vec<BookView>)>,
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
