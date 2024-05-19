use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::actors::book_service::BookData;
use crate::app_state::AppState;
use crate::models;

use super::templates::orderbook::OrderBook;

#[derive(Debug, Deserialize, ToSchema)]
pub struct BookParams {
    /// The id of the book to get messages for
    pub book: u32,
}

pub async fn get(
    State(state): State<AppState>,
    Query(params): Query<BookParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(state, socket, params))
}

async fn handle_socket(mut state: AppState, mut socket: WebSocket, params: BookParams) {
    println!("New websocket connection for book {}", params.book);
    let book = models::book::Book::get(&state.db, params.book)
        .await
        .unwrap();

    let book = OrderBook::from(
        &BookData::new(&state.db, params.book, book.title, book.last_trade_price).await,
    );

    socket
        .send(Message::Text(book.render().unwrap()))
        .await
        .unwrap();

    loop {
        let event = state.book_receive.recv().await.expect("Sender dropped");

        if event.book_id != params.book {
            continue;
        }

        let text = event.render().unwrap();
        if socket.send(Message::Text(text)).await.is_err() {
            return;
        }
    }
}
