use crate::MarketId;
use crate::{Price, Quantity, Side};

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
    /// The market to place the order on.
    pub market: MarketId,
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
        market: MarketId,
        quantity: Quantity,
        price: Price,
        side: Side,
        tif: TimeInForce,
    ) -> Self {
        Self {
            market,
            quantity,
            price,
            side,
            tif,
        }
    }

    #[must_use]
    pub const fn buy(market: MarketId, quantity: Quantity, price: Price, tif: TimeInForce) -> Self {
        Self {
            market,
            quantity,
            price,
            side: Side::Buy,
            tif,
        }
    }

    #[must_use]
    pub const fn sell(market: MarketId, quantity: Quantity, price: Price, tif: TimeInForce) -> Self {
        Self {
            market,
            quantity,
            price,
            side: Side::Sell,
            tif,
        }
    }
}
