use exchange::{BookEvent, Exchange};
use tokio::sync::{broadcast, mpsc};
use tracing::info;

use crate::app_state::current_time_micros;

use super::matcher_request::MatcherRequest;

/// Runs the exchange service.
pub async fn run_market(
    mut exchange: Exchange,
    mut recv: mpsc::Receiver<MatcherRequest>,
    market_data: broadcast::Sender<BookEvent>,
) {
    info!("Starting matching engine...");

    while let Some(msg) = recv.recv().await {
        let timestamp = current_time_micros();

        match msg {
            MatcherRequest::SubmitOrder {
                user,
                order,
                response,
            } => {
                info!(
                    "REQUEST: {timestamp} submit order user_id={user} {:?}",
                    order
                );
                let res = exchange
                    .submit_order(timestamp, user, order.into())
                    .map(|x| x.into());
                if let Ok(event) = res.clone() {
                    market_data.send(event).expect("Receiver dropped");
                }
                response.send(res).expect("Receiver dropped");
            }
            MatcherRequest::CancelOrder {
                user,
                order,
                response,
            } => {
                info!("REQUEST: {timestamp} remove user_id={user} {order}");

                let res = exchange
                    .cancel_order(timestamp, user, order)
                    .map(|x| x.into());
                if let Ok(event) = res.clone() {
                    market_data.send(event).expect("Receiver dropped");
                }
                response.send(res).expect("Receiver dropped");
            }
            MatcherRequest::AddBook { book_id } => {
                exchange.add_book(book_id, 100);
            }
        }
    }
}
