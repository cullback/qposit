use std::collections::HashMap;

use lobster::BookId;
use lobster::OrderBook;
use lobster::Side;
use sqlx::SqlitePool;

use crate::models;

pub mod book_service;
pub mod matcher_request;
pub mod matcher;
pub mod writer;

pub async fn bootstrap_books(
    db: &SqlitePool,
    orders: &[models::order::Order],
) -> HashMap<BookId, OrderBook> {
    let mut books: HashMap<BookId, OrderBook> = models::book::Book::get_active(db)
        .await
        .unwrap()
        .into_iter()
        .map(|x| (x.id, OrderBook::default()))
        .collect();

    for order in orders {
        let book = books.get_mut(&order.book_id).unwrap();
        let order2 = lobster::Order::new(
            order.id,
            order.remaining,
            order.price,
            Side::new(order.is_buy),
        );
        assert!(book.add(order2).is_empty());
    }
    books
}
