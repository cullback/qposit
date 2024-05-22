use crate::{Order, OrderId, Price, Quantity, Side};

use crate::{BookId, Tick, Timestamp, UserId};

/// Book update event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookEvent {
    /// The timestamp this event ocurred at.
    pub time: Timestamp,
    /// Per-book sequence number of this event.
    /// Increases by one for every event.
    pub tick: Tick,
    /// The market this event ocurred on.
    pub book: BookId,
    /// The user that caused the event.
    pub user: UserId,
    /// The type of action that ocurred.
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Add(Order),
    Remove { id: OrderId },
    Resolve { price: Price },
}

impl BookEvent {
    #[must_use]
    pub const fn buy(
        time: Timestamp,
        tick: Tick,
        book: BookId,
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
        book: BookId,
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
        book: BookId,
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
    pub const fn resolve(time: Timestamp, tick: Tick, book: BookId, price: Price) -> Self {
        Self {
            time,
            tick,
            book,
            user: 0,
            action: Action::Resolve { price },
        }
    }
}
