use std::collections::HashMap;

use exchange::BookId;
use orderbook::DefaultBook;
use sqlx::SqlitePool;
use orderbook::Book;

use crate::models;

pub mod book_service;
pub mod matcher;
pub mod matcher_request;
pub mod writer;

pub async fn bootstrap_books(db: &SqlitePool, orders: &[models::order::Order]) -> HashMap<BookId, DefaultBook> {
    let mut books = HashMap::new();

    for book in models::book::Book::get_active(db).await.unwrap() {
        books.insert(book.id, DefaultBook::default());
    }

    for order in orders {
        let book = books.get_mut(&order.book_id).unwrap();
        if order.is_buy {
            assert!(book.buy(order.id, order.remaining, order.price).is_empty());
        } else {
            assert!(book.sell(order.id, order.remaining, order.price).is_empty());
        }
    }
    books
}
