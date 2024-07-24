use crate::{Balance, Order, OrderId, Price, Quantity, Side};

use crate::{MarketId, Tick, Timestamp, UserId};

/// Book update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketUpdate {
    /// The timestamp this update ocurred at.
    pub time: Timestamp,
    /// Per-book sequence number of this event.
    /// Increases by one for every event.
    pub tick: Tick,
    /// The market this event ocurred on.
    pub book: MarketId,
    /// The user that caused the event.
    pub user: UserId,
    /// The type of action that ocurred.
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Add event.
    Add(Order),
    /// Remove event.
    Remove { id: OrderId },
    /// Event was resolved.
    Resolve {
        /// The price the event was resolved to.
        price: Price
    },
    /// A new event was added.
    AddMarket,
    Deposit { amount: Balance },
}

impl MarketUpdate {
    #[must_use]
    pub const fn buy(
        time: Timestamp,
        tick: Tick,
        book: MarketId,
        user: UserId,
        id: OrderId,
        quantity: Quantity,
        price: Price,
    ) -> Self {
        Self {
            time,
            tick,
            book,
            user,
            action: Action::Add(Order::new(id, quantity, price, Side::Buy)),
        }
    }

    #[must_use]
    pub const fn sell(
        time: Timestamp,
        tick: Tick,
        book: MarketId,
        user: UserId,
        id: OrderId,
        quantity: Quantity,
        price: Price,
    ) -> Self {
        Self {
            time,
            tick,
            book,
            user,
            action: Action::Add(Order::new(id, quantity, price, Side::Sell)),
        }
    }

    /// Constructs a remove event.
    #[must_use]
    pub const fn remove(
        time: Timestamp,
        tick: Tick,
        book: MarketId,
        user: UserId,
        id: OrderId,
    ) -> Self {
        Self {
            time,
            tick,
            book,
            user,
            action: Action::Remove { id },
        }
    }

    /// Constructs a resolve event.
    #[must_use]
    pub const fn resolve(
        time: Timestamp,
        tick: Tick,
        book: MarketId,
        user: UserId,
        price: Price,
    ) -> Self {
        Self {
            time,
            tick,
            book,
            user,
            action: Action::Resolve { price },
        }
    }

    #[must_use]
    pub const fn add_book(time: Timestamp, book: MarketId) -> Self {
        Self {
            time,
            tick: 0,
            book,
            user: 0,
            action: Action::AddMarket,
        }
    }

    #[must_use]
    pub const fn deposit(time: Timestamp, user: UserId, amount: Balance) -> Self {
        Self {
            time,
            tick: 0,
            book: 0,
            user,
            action: Action::Deposit { amount },
        }
    }
}
