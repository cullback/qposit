//! Tracks state of all books and streams price-level orderbooks.
//!
//! Every BookEvent represents a change in the order book state that needs to be
//! broadcast to all clients.
//!
//! - volume
//! - last price
//! - order book state
use exchange::{Action, BookEvent, BookId};
use orderbook::Book;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::models;
use crate::pages::OrderBook;

use super::bootstrap_books;

pub struct BookService {
    books: HashMap<BookId, orderbook::DefaultBook>,
}

impl BookService {
    pub async fn new(db: &SqlitePool) -> Self {
        let orders = models::order::Order::get_open_orders(db).await.unwrap();
        let books = bootstrap_books(db, &orders).await;
        Self { books }
    }

    /// Updates the price-by-level for a book
    /// Every add / remove changes the quantity on a level and we send a message
    fn update_levels(&self, book_id: BookId) -> OrderBook {
        let book = self.books.get(&book_id).unwrap();
        OrderBook::from_orders(book.bids(), book.asks())
    }

    fn on_event(&mut self, event: BookEvent) -> OrderBook {
        let book = self.books.get_mut(&event.book).unwrap();
        match event.action {
            Action::Add {
                id,
                quantity,
                price,
                is_buy,
            } => {
                if is_buy {
                    book.buy(id, quantity, price);
                } else {
                    book.sell(id, quantity, price);
                }
            }
            Action::Remove { id } => {
                assert!(book.remove(id));
            }
        }
        let book = self.update_levels(event.book);
        book
    }
}

/// Starts the book service.
pub async fn run_book_service(
    db: SqlitePool,
    mut feed: broadcast::Receiver<BookEvent>,
    book_stream: broadcast::Sender<OrderBook>,
) {
    info!("Starting book service...");
    let mut state = BookService::new(&db).await;

    while let Ok(event) = feed.recv().await {
        let orderbook = state.on_event(event);
        book_stream.send(orderbook).unwrap();
    }
}
