use askama::Template;
use lobster::MarketId;
use lobster::{Order, Price, Quantity};

use crate::actors::book_service::MarketData;

use super::{display_price, format_balance_to_dollars, format_price_to_string};

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
pub struct OrderBook {
    pub market_id: MarketId,
    pub display_price: String,
    pub volume: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

impl From<&MarketData> for OrderBook {
    fn from(market: &MarketData) -> Self {
        let volume = format_balance_to_dollars(market.volume);
        let display_price = display_price(
            market.best_bid,
            market.best_ask,
            market.last_price,
            market.outcome,
        );
        Self {
            market_id: market.market_id,
            display_price,
            volume,
            bids: do_side(market.book.bids()),
            asks: do_side(market.book.asks()),
        }
    }
}
