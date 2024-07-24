//! Wrapper json / api type.
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::app_state::AppState;

/// Book update market.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BookUpdate {
    /// The timestamp this market ocurred at.
    pub time: i64,
    /// Per-book tick sequence number of this market.
    pub tick: u32,
    /// The market associated with this book update.
    pub market: u32,
    /// The user that caused the market. 0 implies unknown.
    pub user: u32,
    /// The type of action that ocurred.
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Add {
        id: i64,
        quantity: u32,
        price: u16,
        is_buy: bool,
    },
    Remove {
        id: i64,
    },
    Resolve {
        price: u16,
    },
    AddMarket,
    Deposit {
        amount: i64,
    },
}

impl From<lobster::MarketUpdate> for BookUpdate {
    fn from(market: lobster::MarketUpdate) -> Self {
        Self {
            time: market.time,
            tick: market.tick,
            market: market.book,
            user: market.user,
            action: match market.action {
                lobster::Action::Add(order) => Action::Add {
                    id: order.id,
                    quantity: order.quantity,
                    price: order.price,
                    is_buy: order.side.is_buy(),
                },
                lobster::Action::Remove { id } => Action::Remove { id },
                lobster::Action::Resolve { price } => Action::Resolve { price },
                lobster::Action::AddMarket => Action::AddMarket,
                lobster::Action::Deposit { amount } => Action::Deposit { amount },
            },
        }
    }
}

/// Subscribe to event data feed.
#[utoipa::path(
    get,
    path = "/api/v1/feed",
    responses(
        (status = 200, description = "Subscribe to event data feed")
    )
)]
pub async fn get(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_socket(state, socket))
}

async fn handle_socket(mut state: AppState, mut socket: WebSocket) {
    loop {
        let update = state.feed_receive.recv().await.expect("Sender dropped");
        let update = BookUpdate::from(update);
        let text = serde_json::to_string(&update).expect("failed to serialize");
        socket.send(Message::Text(text)).await.unwrap();
    }
}
