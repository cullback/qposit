//! Tracks state of all books and streams price-level orderbooks.
//!
//! Every BookEvent represents a change in the order book state that needs to be
//! broadcast to all clients.
//!
//! - volume
//! - last price
//! - order book state
use exchange::{Action, BookEvent, BookId, Tick, UserId};
use kanal::{AsyncReceiver, AsyncSender};
use orderbook::{Book, Order};
use orderbook::{OrderId, Price, Quantity};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::pages::{OrderBook, PriceLevel};

/// Computes the price levels for an order book.
///
/// # Arguments
///
/// - `orders`: iterable of orders sorted from best price to worst price.
fn do_side<'a>(orders: impl IntoIterator<Item = &'a Order>) -> Vec<PriceLevel> {
    let mut price_levels: Vec<PriceLevel> = Vec::new();
    let mut current_price: Option<Price> = None;
    let mut level_quantity = 0;
    let mut cumulative_value = 0;

    for order in orders {
        if current_price == Some(order.price) {
            level_quantity += order.quantity;
            cumulative_value += order.quantity * Quantity::from(order.price);
        } else {
            if let Some(price) = current_price {
                price_levels.push(PriceLevel {
                    price: format!("{:.2}", f32::from(price) / 100.0),
                    quantity: level_quantity.to_string(),
                    value: format!("{:.2}", f64::from(cumulative_value) / 10000.0),
                });
            }
            level_quantity = order.quantity;
            cumulative_value += order.quantity * Quantity::from(order.price);
            current_price = Some(order.price);
        }
    }

    if let Some(price) = current_price {
        price_levels.push(PriceLevel {
            price: format!("{:.2}", f32::from(price) / 100.0),
            quantity: level_quantity.to_string(),
            value: format!("{:.2}", f64::from(cumulative_value) / 10000.0),
        });
    }

    price_levels
}

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

    /// Updates the price-by-level for a book
    /// Every add / remove changes the quantity on a level and we send a message
    fn update_levels(&self, book: BookId) -> OrderBook {
        let book = self.books.get(&book).unwrap();

        let bids = do_side(book.bids());
        let asks = do_side(book.asks());

        OrderBook { bids, asks }
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
pub async fn run_book_service(feed: AsyncReceiver<BookEvent>,
    book_stream: AsyncSender<OrderBook>,
) {
    info!("Starting book service...");
    let mut state = BookService::new();

    while let Ok(event) = feed.recv().await {
        let orderbook = state.on_event(event);
        book_stream.send(orderbook).await.unwrap();
    }
}
