//! Tracks state of all books and streams price-level orderbooks.
//!
//! Every BookEvent represents a change in the order book state that needs to be
//! broadcast to all clients.
//!
//! - volume
//! - last price
//! - order book state
use exchange::{Action, BookEvent, BookId, Tick, UserId};
use orderbook::{Book, Order};
use orderbook::{OrderId, Price, Quantity};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

type PriceLevel = (Price, Quantity, Quantity);

pub struct BookService {
    books: HashMap<BookId, orderbook::DefaultBook>,
    order_owner: HashMap<OrderId, UserId>,
}

impl BookService {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
            order_owner: HashMap::new(),
        }
    }

    /// Computes price levels from best to worst.
    /// Orders should be sorted from best price to worst price.
    fn do_side<'a>(orders: impl IntoIterator<Item = &'a Order>) -> Vec<PriceLevel> {
        let mut price_levels: Vec<PriceLevel> = Vec::new();
        let mut current_price: Option<Price> = None;
        let mut total_quantity = 0;
        let mut cumulative_value = 0;

        for order in orders {
            if current_price == Some(order.price) {
                total_quantity += order.quantity;
                cumulative_value += order.quantity * Quantity::from(order.price);
            } else {
                if let Some(price) = current_price {
                    price_levels.push((price, total_quantity, cumulative_value));
                }
                total_quantity = order.quantity;
                cumulative_value += order.quantity * Quantity::from(order.price);
                current_price = Some(order.price);
            }
        }
        price_levels
    }

    /// Updates the price-by-level for a book
    /// Every add / remove changes the quantity on a level and we send a message
    fn update_levels(&self, book: BookId) {
        let book = self.books.get(&book).unwrap();

        let bids = Self::do_side(book.bids());
        let asks = Self::do_side(book.asks());
    }

    fn on_event(&mut self, event: BookEvent) {
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
        self.update_levels(event.book);
    }
}

/// Starts the book service.
pub async fn run_book_service(mut feed: broadcast::Receiver<BookEvent>) {
    info!("Starting book service...");
    let mut state = BookService::new();

    while let Ok(event) = feed.recv().await {
        state.on_event(event);
    }
}
