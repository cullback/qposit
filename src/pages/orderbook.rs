use std::collections::HashSet;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use exchange::BookId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::actors::book_service::BookData;
use crate::app_state::AppState;
use crate::models;

use super::templates::orderbook::OrderBook;

#[derive(Debug, Deserialize, ToSchema)]
pub struct BookParams {
    /// The id of the book to get messages for
    pub books: String,
}

pub async fn get(
    State(state): State<AppState>,
    Query(params): Query<BookParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(state, socket, params))
}

async fn handle_socket(mut state: AppState, mut socket: WebSocket, params: BookParams) {
    let Ok(books) = params
        .books
        .split(',')
        .map(|x| x.parse())
        .collect::<Result<HashSet<BookId>, _>>()
    else {
        return;
    };

    for &book_id in &books {
        let book = models::book::Book::get(&state.db, book_id).await.unwrap();

        let book = OrderBook::from(
            &BookData::new(&state.db, book_id, book.title, book.last_trade_price).await,
        );

        let text = book.render().unwrap();
        if socket.send(Message::Text(text)).await.is_err() {
            return;
        }
    }

    loop {
        let event = state.book_receive.recv().await.expect("Sender dropped");

        if !books.contains(&event.book_id) {
            continue;
        }

        let text = event.render().unwrap();
        if socket.send(Message::Text(text)).await.is_err() {
            return;
        }
    }
}
