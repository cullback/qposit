//! # Book Service
//!
//! The book service tracks state of all books and streams price-level orderbooks
//! to the front end UI.
//!
//! Every BookEvent represents a change in the order book state that needs to be
//! broadcast to all clients.
//!
//! - volume
//! - last price, mid price
//! - best bid, best ask
//! - order book state
//!
//! TODO: update state more efficiently
//! - track price levels individually instead of updating everything on every event.
use lobster::{Action, BookEvent, BookId};
use lobster::{Order, Price, Side};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::models;
use crate::models::book::Book;

/// Snapshot of the latest order book data to be rendered.
#[derive(Debug, Clone)]
pub struct BookData {
    pub book_id: BookId,
    pub inner: lobster::OrderBook,
    pub best_bid_price: Option<Price>,
    pub best_ask_price: Option<Price>,
    pub last_price: Option<Price>,
    pub volume: u64,
}

impl BookData {
    pub async fn new(db: &SqlitePool, book_id: BookId, last_trade_price: Option<Price>) -> Self {
        let orders = models::order::Order::get_open_for_book(db, book_id)
            .await
            .unwrap();
        let mut book = lobster::OrderBook::default();
        for order in orders {
            let order2 = Order::new(
                order.id,
                order.remaining,
                order.price,
                Side::new(order.is_buy),
            );
            assert!(book.add(order2).is_empty());
        }

        let volume = Book::get_volume(db, book_id).await.unwrap();

        Self {
            book_id,
            best_bid_price: book.best_bid().map(|x| x.price),
            best_ask_price: book.best_ask().map(|x| x.price),
            inner: book,
            last_price: last_trade_price,
            volume,
        }
    }

    pub fn new_default(book_id: BookId) -> Self {
        Self {
            book_id,
            inner: lobster::OrderBook::default(),
            best_bid_price: None,
            best_ask_price: None,
            last_price: None,
            volume: 0,
        }
    }

    pub fn on_event(&mut self, event: BookEvent) {
        match event.action {
            Action::Add(order) => {
                let fills = self.inner.add(order);
                for fill in fills {
                    self.volume += u64::from(fill.quantity) * u64::from(fill.price);
                    self.last_price = Some(fill.price);
                }
                self.best_bid_price = self.inner.best_bid().map(|x| x.price);
                self.best_ask_price = self.inner.best_ask().map(|x| x.price);
            }
            Action::Remove { id } => {
                assert!(self.inner.remove(id).is_some());
                self.best_bid_price = self.inner.best_bid().map(|x| x.price);
                self.best_ask_price = self.inner.best_ask().map(|x| x.price);
            }
            Action::Resolve { price } => {
                self.last_price = Some(price);
            }
            Action::AddBook => {}
        }
    }
}

struct BookService {
    books: HashMap<BookId, BookData>,
}

impl BookService {
    pub async fn new(db: &SqlitePool) -> Self {
        let mut books = HashMap::new();
        for book in models::book::Book::get_active(db).await.unwrap() {
            let book_data = BookData::new(db, book.id, book.last_trade_price).await;
            books.insert(book.id, book_data);
        }

        Self { books }
    }

    fn on_event(&mut self, event: BookEvent) -> BookData {
        if matches!(event.action, Action::AddBook) {
            self.books
                .insert(event.book, BookData::new_default(event.book));
        }
        let book = self.books.get_mut(&event.book).unwrap();
        book.on_event(event);
        book.clone()
    }
}

pub fn start_book_service(
    db: SqlitePool,
    mut feed: broadcast::Receiver<BookEvent>,
    book_stream: broadcast::Sender<BookData>,
) {
    tokio::spawn({
        async move {
            info!("Starting book service...");
            let mut state = BookService::new(&db).await;

            while let Ok(event) = feed.recv().await {
                let orderbook = state.on_event(event);
                book_stream.send(orderbook).unwrap();
            }
        }
    });
}
