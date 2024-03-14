use exchange::{BookEvent, BookId, Exchange, OrderRequest, RejectReason, UserId};
use orderbook::OrderId;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::info;

use crate::app_state::timestamp_micros;

/// A request to the database engine.
/// These messages are used to communicate from the endpoints to the matching engine service.
#[derive(Debug)]
pub enum MatcherRequest {
    SubmitOrder {
        user: UserId,
        order: OrderRequest,
        /// Response to the client
        response: oneshot::Sender<Result<BookEvent, RejectReason>>,
    },
    CancelOrder {
        user: UserId,
        order: OrderId,
        /// Response to the client
        response: oneshot::Sender<Result<BookEvent, RejectReason>>,
    },
    AddBook {
        book_id: BookId,
    },
}

/// Runs the exchange service.
pub async fn run_market(
    mut exchange: Exchange,
    mut recv: mpsc::Receiver<MatcherRequest>,
    market_data: broadcast::Sender<BookEvent>,
) {
    info!("Starting matching engine...");

    while let Some(msg) = recv.recv().await {
        let timestamp = timestamp_micros();

        match msg {
            MatcherRequest::SubmitOrder {
                user,
                order,
                response,
            } => {
                info!("REQUEST: {timestamp} add user_id={user} {:?}", order);
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
