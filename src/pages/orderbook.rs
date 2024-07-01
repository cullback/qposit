use std::collections::HashSet;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use lobster::EventId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::app_state::AppState;

use super::templates::orderbook::OrderBook;

#[derive(Debug, Deserialize, ToSchema)]
pub struct EventParams {
    /// The id of the book to get messages for
    pub events: String,
}

pub async fn get(
    State(state): State<AppState>,
    Query(params): Query<EventParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(state, socket, params))
}

async fn handle_socket(mut state: AppState, mut socket: WebSocket, params: EventParams) {
    let Ok(events) = params
        .events
        .split(',')
        .map(|x| x.parse())
        .collect::<Result<HashSet<EventId>, _>>()
    else {
        return;
    };

    loop {
        let event = state.book_receive.recv().await.expect("Sender dropped");

        if !events.contains(&event.event_id) {
            continue;
        }

        let text = OrderBook::from(&event).render().unwrap();
        if socket.send(Message::Text(text)).await.is_err() {
            return;
        }
    }
}
