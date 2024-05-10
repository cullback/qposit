use askama::Template;
use orderbook::{Order, Price, Quantity};

#[derive(Debug, Clone)]
struct PriceLevel {
    pub price: String,
    pub quantity: String,
    pub value: String,
}

/// Computes the price levels for an order book.
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

#[derive(Template, Debug, Clone)]
#[template(path = "orderbook.html")]
pub struct OrderBook {
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

impl OrderBook {
    /// Orders should be sorted by prices ascending.
    pub fn from_orders(
        bids: impl IntoIterator<Item = Order>,
        asks: impl IntoIterator<Item = Order>,
    ) -> Self {
        Self {
            bids: do_side(bids),
            asks: do_side(asks),
        }
    }
}
