use crate::{Balance, Order, OrderId, Price};

use crate::{MarketId, Tick, Timestamp, UserId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketUpdate {
    AddOrder {
        timestamp: Timestamp,
        tick: Tick,
        market: MarketId,
        user: UserId,
        order: Order,
    },
    RemoveOrder {
        timestamp: Timestamp,
        tick: Tick,
        market: MarketId,
        user: UserId,
        /// The id of the order to remove
        id: OrderId,
    },
    ResolveMarket {
        timestamp: Timestamp,
        tick: Tick,
        market: MarketId,
        /// The price the market was resolved to.
        price: Price,
    },
    /// A new event was added.
    AddMarket {
        timestamp: Timestamp,
        tick: Tick,
        market: MarketId,
    },
    Deposit {
        timestamp: Timestamp,
        user: UserId,
        amount: Balance,
    },
}

