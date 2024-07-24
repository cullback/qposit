use super::{OrderId, Price, Quantity};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Fill {
    /// The order id of the maker order.
    pub id: OrderId,
    /// The number of contracts matched.
    pub quantity: Quantity,
    /// The price the order was matched at.
    pub price: Price,
    /// `true` if the matching order has no size remaining.
    pub done: bool,
}

impl Fill {
    /// Constructs a new fill.
    #[must_use]
    pub const fn new(id: OrderId, quantity: Quantity, price: Price, done: bool) -> Self {
        Self {
            id,
            quantity,
            price,
            done,
        }
    }
}
