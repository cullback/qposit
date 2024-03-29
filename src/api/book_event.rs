//! Wrapper json / api type.
use orderbook::{OrderId, Price, Size};
use serde::Serialize;
use utoipa::ToSchema;

/// Book update event.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct BookEvent {
    /// The timestamp this event ocurred at.
    pub time: i64,
    /// Per-book tick sequence number of this event.
    pub tick: u32,
    /// The market this event ocurred on.
    pub book: u32,
    /// The user that caused the event. 0 implies unknown.
    pub user: u32,
    /// The type of action that ocurred.
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Action {
    Add {
        id: u64,
        size: u32,
        price: u16,
        is_buy: bool,
    },
    Remove {
        id: u64,
    },
}

impl From<exchange::BookEvent> for BookEvent {
    fn from(event: exchange::BookEvent) -> Self {
        Self {
            time: event.time,
            tick: event.tick,
            book: event.book,
            user: event.user,
            action: match event.action {
                exchange::Action::Add {
                    id,
                    size,
                    price,
                    is_buy,
                } => Action::Add {
                    id,
                    size,
                    price,
                    is_buy,
                },
                exchange::Action::Remove { id } => Action::Remove { id },
            },
        }
    }
}
