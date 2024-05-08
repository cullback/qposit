use askama::Template;
use axum::extract::{Query, State};
use axum::response::{sse::Event, Sse};
use exchange::BookId;
use futures::Stream;
use orderbook::Order;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use utoipa::ToSchema;

use crate::app_state::AppState;
use crate::models;

use super::templates::orderbook::OrderBook;

pub async fn book_snapshot(db: &SqlitePool, book_id: BookId) -> OrderBook {
    let orders = models::order::Order::get_open_for_book(db, book_id)
        .await
        .unwrap();

    let bids = orders
        .iter()
        .filter(|o| o.is_buy)
        .map(|x| Order::from(x))
        .rev();
    let asks = orders.iter().filter(|o| !o.is_buy).map(|x| Order::from(x));

    let orderbook = OrderBook::from_orders(bids, asks);
    orderbook
}

/// Request for a new order.
#[derive(Debug, Deserialize, ToSchema)]
pub struct BookParams {
    /// The id of the book to get messages for
    pub book: u32,
}

pub async fn sse_handler(
    State(state): State<AppState>,
    Query(params): Query<BookParams>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let book = book_snapshot(&state.db, params.book.into()).await;
    let receiver_stream = BroadcastStream::new(state.book_receive)
        .map(|msg| Ok(Event::default().data(msg.unwrap().render().unwrap())));
    let stream = tokio_stream::once(Ok(Event::default().data(book.render().unwrap())))
        .chain(receiver_stream);

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
