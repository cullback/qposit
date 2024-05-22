use crate::{Price, Quantity, Side};
use crate::BookId;

/// Time in force for the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum TimeInForce {
    /// Good until canceled
    GTC,
    /// Immediate or cancel
    IOC,
    /// Post-only. Order is not added to the book if it is marketable.
    /// Implies it is also GTC.
    POST,
}

/// Request for a new order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderRequest {
    /// The book to place the order on.
    pub book: BookId,
    /// The number of contracts to buy or sell.
    pub quantity: Quantity,
    /// The price to buy or sell at.
    pub price: Price,
    /// Whether the order is a buy or sell.
    pub side: Side,
    /// Order type.
    pub tif: TimeInForce,
}

impl OrderRequest {
    /// Constructs a new order request.
    #[must_use]
    pub const fn new(
        book: BookId,
        quantity: Quantity,
        price: Price,
        side: Side,
        tif: TimeInForce,
    ) -> Self {
        Self {
            book,
            quantity,
            price,
            side,
            tif,
        }
    }

    #[must_use]
    pub const fn buy(book: BookId, quantity: Quantity, price: Price, tif: TimeInForce) -> Self {
        Self {
            book,
            quantity,
            price,
            side: Side::Buy,
            tif,
        }
    }

    #[must_use]
    pub const fn sell(book: BookId, quantity: Quantity, price: Price, tif: TimeInForce) -> Self {
        Self {
            book,
            quantity,
            price,
            side: Side::Sell,
            tif,
        }
    }
}
