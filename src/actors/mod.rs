use std::collections::HashMap;

use exchange::BookId;
use orderbook::Book;
use orderbook::DefaultBook;
use orderbook::Side;
use sqlx::SqlitePool;

use crate::models;

pub mod book_service;
pub mod matcher;
pub mod matcher_request;
pub mod writer;

pub async fn bootstrap_books(
    db: &SqlitePool,
    orders: &[models::order::Order],
) -> HashMap<BookId, DefaultBook> {
    let mut books: HashMap<BookId, DefaultBook> = models::book::Book::get_active(db)
        .await
        .unwrap()
        .into_iter()
        .map(|x| (x.id, DefaultBook::default()))
        .collect();

    for order in orders {
        let book = books.get_mut(&order.book_id).unwrap();
        let order2 = orderbook::Order::new(
            order.id,
            order.remaining,
            order.price,
            Side::new(order.is_buy),
        );
        assert!(book.add(order2).is_empty());
    }
    books
}
