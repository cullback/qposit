use super::{OrderId, Price, Quantity, Side};

/// An order in the order book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Order {
    /// The order id.
    pub id: OrderId,
    /// The remaining quantity in the order.
    pub quantity: Quantity,
    /// The price of this order.
    pub price: Price,
    /// The side of this order.
    pub side: Side,
}

impl Order {
    #[must_use]
    pub const fn new(id: OrderId, quantity: Quantity, price: Price, side: Side) -> Self {
        Self {
            id,
            quantity,
            price,
            side,
        }
    }

    #[must_use]
    pub const fn buy(id: OrderId, quantity: Quantity, price: Price) -> Self {
        Self {
            id,
            quantity,
            price,
            side: Side::Buy,
        }
    }

    #[must_use]
    pub const fn sell(id: OrderId, quantity: Quantity, price: Price) -> Self {
        Self {
            id,
            quantity,
            price,
            side: Side::Sell,
        }
    }
}
