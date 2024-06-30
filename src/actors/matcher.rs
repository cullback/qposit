use std::collections::HashMap;

use lobster::{Balance, Side, UserId};
use lobster::{BookEvent, BookId, Exchange};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

use crate::app_state::current_time_micros;

use super::matcher_request::MatcherRequest;

use crate::models::{book::Book, order::Order, position::Position, user::User};

/// Initializes the in-memory exchange data from the database.
async fn bootstrap_exchange(db: &SqlitePool) -> Exchange {
    let next_order_id = Order::get_next_order_id(db).await;

    let mut balances: HashMap<UserId, Balance> = HashMap::new();
    for user in User::get_with_nonzero_balances(db).await.unwrap() {
        balances.insert(user.id, user.balance);
    }

    let mut books: Vec<BookId> = Vec::new();
    for book in Book::get_active(db).await.unwrap() {
        books.push(book.id);
    }

    let mut positions: HashMap<(UserId, BookId), i32> = HashMap::new();
    for position in Position::get_non_zero(db).await.unwrap() {
        positions.insert((position.user_id, position.book_id), position.position);
    }

    let mut orders: Vec<(UserId, BookId, lobster::Order)> = Vec::new();
    for order_record in Order::get_open_orders(db).await.unwrap() {
        let order = lobster::Order::new(
            order_record.id,
            order_record.quantity,
            order_record.price,
            Side::new(order_record.is_buy),
        );
        orders.push((order_record.user_id, order_record.book_id, order));
    }

    let engine = lobster::Exchange::from_state(
        next_order_id,
        &balances,
        &positions,
        orders.as_slice(),
        books.as_slice(),
    );

    engine
}

pub fn start_matcher_service(
    db: SqlitePool,
    mut recv: mpsc::Receiver<MatcherRequest>,
    market_data: broadcast::Sender<BookEvent>,
) {
    tokio::spawn({
        async move {
            info!("Starting matching engine...");
            let mut exchange = bootstrap_exchange(&db).await;

            while let Some(msg) = recv.recv().await {
                let timestamp = current_time_micros();
                info!("REQUEST: {timestamp} request: {msg:?}");
                match msg {
                    MatcherRequest::SubmitOrder {
                        user,
                        order,
                        response,
                    } => {
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
                        let res = exchange.cancel_order(timestamp, user, order);
                        if let Ok(event) = res.clone() {
                            market_data.send(event).expect("Receiver dropped");
                        }
                        response.send(res).expect("Receiver dropped");
                    }
                    MatcherRequest::AddBook { book_id } => {
                        let event = exchange.add_book(timestamp, book_id).unwrap();
                        market_data.send(event).expect("Receiver dropped");
                    }
                    MatcherRequest::Deposit { user, amount } => {
                        exchange.deposit(user, amount);
                    }
                    MatcherRequest::Resolve {
                        book_id,
                        price,
                        response,
                    } => {
                        let event = exchange.resolve(timestamp, book_id, price);
                        response.send(event).expect("Receiver dropped");
                    }
                }
            }
        }
    });
}
