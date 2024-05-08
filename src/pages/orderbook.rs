use askama::Template;
use axum::extract::{Query, State};
use axum::response::{sse::Event, Sse};
use futures::Stream;
use futures::StreamExt;
use serde::Deserialize;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::BroadcastStream;
use utoipa::ToSchema;

use crate::app_state::AppState;

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

    // Convert cmd_receive into a stream
    let receiver_stream = BroadcastStream::new(state.book_receive).filter_map(|msg| async move {
        Some(Ok(Event::default().data(msg.unwrap().render().unwrap())))
    });

    Sse::new(receiver_stream).keep_alive(axum::response::sse::KeepAlive::default())
}
