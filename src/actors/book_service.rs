//! Tracks state of all books and streams price-level orderbooks.
//!
//! Every BookEvent represents a change in the order book state that needs to be
//! broadcast to all clients.
//!
//! - volume
//! - last price
//! - order book state
//!
//! TODO: update state more efficiently
//! - track price levels individually instead of updating everything on every event.
use exchange::{Action, BookEvent, BookId};
use orderbook::{Book, DefaultBook, Price};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::models;
use crate::pages::OrderBook;

pub struct BookData {
    pub book_id: BookId,
    pub title: String,
    pub inner: orderbook::DefaultBook,
    pub last_price: Option<Price>,
    pub volume: u64,
}

impl BookData {
    async fn get_volume_for_book(db: &SqlitePool, book_id: BookId) -> u64 {
        let (volume,) =
            sqlx::query_as::<_, (i64,)>("SELECT SUM(quantity) FROM trade WHERE book_id = ?")
                .bind(book_id)
                .fetch_one(db)
                .await
                .unwrap();
        u64::try_from(volume).unwrap()
    }

    pub async fn new(
        db: &SqlitePool,
        book_id: BookId,
        title: String,
        last_trade_price: Option<Price>,
    ) -> Self {
        let orders = models::order::Order::get_open_for_book(db, book_id)
            .await
            .unwrap();
        let mut book = DefaultBook::default();
        for order in orders {
            assert!(book
                .add(order.id, order.remaining, order.price, order.is_buy)
                .is_empty());
        }

        let volume = Self::get_volume_for_book(db, book_id).await;

        BookData {
            book_id,
            title,
            inner: book,
            last_price: last_trade_price,
            volume: volume as u64,
        }
    }

    pub fn on_event(&mut self, event: BookEvent) {
        match event.action {
            Action::Add {
                id,
                quantity,
                price,
                is_buy,
            } => {
                let fills = self.inner.add(id, quantity, price, is_buy);
                for fill in fills {
                    self.volume += u64::from(fill.quantity);
                    self.last_price = Some(fill.price);
                }
            }
            Action::Remove { id } => {
                assert!(self.inner.remove(id));
            }
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
            let book_data = BookData::new(db, book.id, book.title, book.last_trade_price).await;
            books.insert(book.id, book_data);
        }

        Self { books }
    }

    fn on_event(&mut self, event: BookEvent) -> OrderBook {
        let book = self.books.get_mut(&event.book).unwrap();
        book.on_event(event);
        (&*book).into()
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
