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
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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
                    quantity,
                    price,
                    is_buy,
                } => Action::Add {
                    id,
                    quantity,
                    price,
                    is_buy,
                },
                exchange::Action::Remove { id } => Action::Remove { id },
            },
        }
    }
}

#[utoipa::path(
    get,
    path = "/ws",
    responses(
        (status = 200, description = "Subscribe to market data feed")
    )
)]
pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_socket(state, socket))
}

async fn handle_socket(mut state: AppState, mut socket: WebSocket) {
    println!("new websocket connection");
    loop {
        let event = state.feed_receive.recv().await.expect("Sender dropped");
        let event = BookEvent::from(event);
        let text = serde_json::to_string(&event).expect("failed to serialize");
        socket.send(Message::Text(text)).await.unwrap();
    }
}
