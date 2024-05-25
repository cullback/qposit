use askama::Template;
use lobster::BookId;
use lobster::{Order, Price, Quantity};

use crate::actors::book_service::BookData;

#[derive(Debug, Clone)]
struct PriceLevel {
    pub price: String,
    pub quantity: String,
    pub value: String,
}

impl PriceLevel {
    pub fn new(price: Price, quantity: Quantity, cumulative_value: u32) -> Self {
        Self {
            price: format!("{:.2}", f32::from(price) / 100.0),
            quantity: quantity.to_string(),
            value: format!("{:.2}", f64::from(cumulative_value) / 10000.0),
        }
    }
}

/// Computes the price levels for a side of an order book.
///
/// # Arguments
///
/// - `orders`: iterable of orders sorted from best price to worst price.
fn do_side(orders: impl IntoIterator<Item = Order>) -> Vec<PriceLevel> {
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
                price_levels.push(PriceLevel::new(price, level_quantity, cumulative_value));
            }
            level_quantity = order.quantity;
            cumulative_value += order.quantity * Quantity::from(order.price);
            current_price = Some(order.price);
        }
    }

    if let Some(price) = current_price {
        price_levels.push(PriceLevel::new(price, level_quantity, cumulative_value));
    }
    price_levels
}

#[derive(Template, Debug, Clone)]
#[template(path = "orderbook.html")]
pub struct OrderBook {
    pub book_id: BookId,
    title: String,
    last_price: String,
    mid_price: String,
    volume: String,
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

impl From<&BookData> for OrderBook {
    fn from(book: &BookData) -> Self {
        let last_price = format!("{:.2}", book.last_price.unwrap_or(0) as f32 / 100.0);
        let volume = format!("{:.2}", book.volume as f32 / 10000.0);
        let mid_price = match (book.best_bid_price, book.best_ask_price) {
            (Some(bid), Some(ask)) => format!("{:.2}", (bid + ask) as f32 / 200.0),
            (Some(bid), None) => format!("{:.2}", bid as f32 / 100.0),
            (None, Some(ask)) => format!("{:.2}", ask as f32 / 100.0),
            _ => "N/A".to_string(),
        };
        Self {
            book_id: book.book_id,
            title: book.title.clone(),
            last_price,
            mid_price,
            volume,
            bids: do_side(book.inner.bids()),
            asks: do_side(book.inner.asks()),
        }
    }
}
