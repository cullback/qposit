use askama::Template;
use lobster::BookId;
use lobster::{Order, Price, Quantity};

use crate::actors::book_service::BookData;

use super::{format_price_to_string, mid_to_string};

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: String,
    pub quantity: String,
    pub value: String,
}

impl PriceLevel {
    pub fn new(price: Price, quantity: Quantity, cumulative_value: u32) -> Self {
        Self {
            price: format_price_to_string(price),
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
pub fn do_side(orders: impl IntoIterator<Item = Order>) -> Vec<PriceLevel> {
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

/// Book data delivered over websocket feed.
/// Needs to be applied against an already rendered book.
#[derive(Template, Debug, Clone)]
#[template(path = "orderbook.html")]
#[allow(dead_code)]
pub struct OrderBook {
    pub book_id: BookId,
    pub last_price: String,
    pub mid_price: String,
    pub volume: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

impl From<&BookData> for OrderBook {
    fn from(book: &BookData) -> Self {
        let last_price = format_price_to_string(book.last_price.unwrap_or(0));
        let volume = format!("{:.2}", book.volume as f32 / 10000.0);
        let mid_price = mid_to_string(book.best_bid_price, book.best_ask_price);
        Self {
            book_id: book.book_id,
            last_price,
            mid_price,
            volume,
            bids: do_side(book.inner.bids()),
            asks: do_side(book.inner.asks()),
        }
    }
}
