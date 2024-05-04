use exchange::{BookEvent, Exchange, OrderRequest, TimeInForce};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

use crate::app_state::current_time_micros;

use super::matcher_request::MatcherRequest;

use crate::models::{book::Book, order::Order, position::Position, user::User};

/// Initializes the in-memory exchange data from the database.
async fn bootstrap_exchange(db: &SqlitePool) -> Exchange {
    let next_order_id = Order::get_next_order_id(db).await;

    let mut engine = Exchange::new(next_order_id);

    for user in User::get_with_nonzero_balances(db).await.unwrap() {
        engine.deposit(user.id, user.balance);
    }

    for book in Book::get_active(db).await.unwrap() {
        engine.add_book(book.id, 100);
    }

    for position in Position::get_non_zero(db).await.unwrap() {
        engine.set_position(position.user_id, position.book_id, position.position);
    }

    for order in Order::get_open_orders(db).await.unwrap() {
        let req = OrderRequest::new(
            order.book_id,
            order.quantity,
            order.price,
            order.is_buy,
            TimeInForce::GTC,
        );
        engine.init_order(order.id, order.user_id, req);
    }

    engine
}

/// Runs the exchange service.
pub async fn run_matcher(
    db: SqlitePool,
    mut recv: mpsc::Receiver<MatcherRequest>,
    market_data: broadcast::Sender<BookEvent>,
) {
    info!("Starting matching engine...");
    let mut exchange = bootstrap_exchange(&db).await;

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
                let res = exchange.submit_order(timestamp, user, order);
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

                let res = exchange.cancel_order(timestamp, user, order);
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
