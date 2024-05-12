use askama::Template;
use axum::extract::{Query, State};
use axum::response::{sse::Event, Sse};
use futures::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use utoipa::ToSchema;

use crate::actors::book_service::BookData;
use crate::app_state::AppState;

use super::templates::orderbook::OrderBook;

#[derive(Debug, Deserialize, ToSchema)]
pub struct BookParams {
    /// The id of the book to get messages for
    pub book: u32,
}

/// Generate a stream for a specific book.
pub async fn sse_handler(
    State(state): State<AppState>,
    Query(params): Query<BookParams>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver_stream = BroadcastStream::new(state.book_receive)
        .map(|msg| Ok(Event::default().data(msg.unwrap().render().unwrap())));

    let book = OrderBook::from(&BookData::new(&state.db, params.book).await);
    let stream = tokio_stream::once(Ok(Event::default().data(book.render().unwrap())))
        .chain(receiver_stream);

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
