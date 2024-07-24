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

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MarketUpdate {
    AddOrder {
        timestamp: i64,
        tick: u32,
        market: u32,
        user: u32,
        id: i64,
        quantity: u32,
        price: u16,
        is_buy: bool,
    },
    RemoveOrder {
        timestamp: i64,
        tick: u32,
        market: u32,
        user: u32,
        /// The id of the order to remove
        id: i64,
    },
    ResolveMarket {
        timestamp: i64,
        tick: u32,
        market: u32,
        /// The price the market was resolved to.
        price: u16,
    },
    /// A new event was added.
    AddMarket {
        timestamp: i64,
        tick: u32,
        market: u32,
    },
    Deposit {
        timestamp: i64,
        user: u32,
        amount: i64,
    },
}

impl From<lobster::MarketUpdate> for MarketUpdate {
    fn from(update: lobster::MarketUpdate) -> Self {
        match update {
            lobster::MarketUpdate::AddOrder {
                timestamp,
                tick,
                market,
                user,
                order,
            } => MarketUpdate::AddOrder {
                timestamp,
                tick,
                market,
                user,
                id: order.id,
                quantity: order.quantity,
                price: order.price,
                is_buy: order.side.is_buy(),
            },
            lobster::MarketUpdate::RemoveOrder {
                timestamp,
                tick,
                market,
                user,
                id,
            } => MarketUpdate::RemoveOrder {
                timestamp,
                tick,
                market,
                user,
                id,
            },
            lobster::MarketUpdate::ResolveMarket {
                timestamp,
                tick,
                market,
                price,
            } => MarketUpdate::ResolveMarket {
                timestamp,
                tick,
                market,
                price,
            },
            lobster::MarketUpdate::AddMarket {
                timestamp,
                tick,
                market,
            } => MarketUpdate::AddMarket {
                timestamp,
                tick,
                market,
            },
            lobster::MarketUpdate::Deposit {
                timestamp,
                user,
                amount,
            } => MarketUpdate::Deposit {
                timestamp,
                user,
                amount: amount.into(),
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
        let update = MarketUpdate::from(update);
        let text = serde_json::to_string(&update).expect("failed to serialize");
        socket.send(Message::Text(text)).await.unwrap();
    }
}
