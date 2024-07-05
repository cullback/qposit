use std::collections::HashSet;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use lobster::MarketId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::app_state::AppState;

use super::templates::market_update::MarketUpdate;

#[derive(Debug, Deserialize, ToSchema)]
pub struct EventParams {
    /// The comma separated ids of the markets to get messages for
    pub markets: String,
}

pub async fn get(
    State(state): State<AppState>,
    Query(params): Query<EventParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(state, socket, params))
}

async fn handle_socket(mut state: AppState, mut socket: WebSocket, params: EventParams) {
    let Ok(markets) = params
        .markets
        .split(',')
        .map(|x| x.parse())
        .collect::<Result<HashSet<MarketId>, _>>()
    else {
        return;
    };

    loop {
        let market = state.book_receive.recv().await.expect("Sender dropped");

        if !markets.contains(&market.market_id) {
            continue;
        }

        let text = MarketUpdate::from(&market).render().unwrap();
        if socket.send(Message::Text(text)).await.is_err() {
            return;
        }
    }
}
