use axum::response::{sse::Event, Sse};
use futures::Stream;
use futures::StreamExt;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (cmd_send, cmd_receive) = mpsc::channel::<String>(32);

    // Spawn a task to send data to the SSE stream
    tokio::spawn(async move {
        // Simulate sending some data
        for i in 0..10 {
            // Send data to the SSE stream
            if let Err(_) = cmd_send.send(format!("<p>hello {i}</p>")).await {
                // Handle error if sending fails
                eprintln!("Error sending data to SSE stream");
                break;
            }
            // Introduce a small delay
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });

    // Convert cmd_receive into a stream
    let receiver_stream =
        ReceiverStream::new(cmd_receive).map(|msg| Ok(Event::default().data(msg)));

    Sse::new(receiver_stream).keep_alive(axum::response::sse::KeepAlive::default())
}
