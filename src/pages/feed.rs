use askama::Template;
use axum::extract::{Query, State};
use axum::response::{sse::Event, Sse};
use futures::Stream;
use futures::StreamExt;
use kanal::ReceiveStream;
use serde::Deserialize;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
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
    // let (cmd_send, cmd_receive) = mpsc::channel::<String>(32);

    // // Spawn a task to send data to the SSE stream
    // tokio::spawn(async move {
    //     // Simulate sending some data
    //     for i in 0..10 {
    //         // Send data to the SSE stream
    //         if let Err(_) = cmd_send.send(format!("<p>hello {i}</p>")).await {
    //             // Handle error if sending fails
    //             eprintln!("Error sending data to SSE stream");
    //             break;
    //         }
    //         // Introduce a small delay
    //         tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    //     }
    // });

    // let receiver_stream =
    // ReceiverStream::new(cmd_receive).map(|msg| Ok(Event::default().data(msg)));

    // let thing = state.book_receive.clone();

    // // Convert cmd_receive into a stream
    // let x = thing
    //     .stream()
    //     .map(|msg| Ok(Event::default().data(msg.render().unwrap())));

    // let (s, r) = kanal::unbounded_async();
    // tokio::spawn(async move {
    //     for i in 0..100 {
    //         s.send(i).await.unwrap();
    //     }
    // });

    let r = state.book_receive;

    let stream = r
        .stream()
        .map(|msg| Ok(Event::default().data(msg.to_string())));

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
