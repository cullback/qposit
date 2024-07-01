use std::collections::HashMap;

use lobster::{Balance, Side, UserId};
use lobster::{BookUpdate, EventId, Exchange};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

use crate::app_state::current_time_micros;

use super::matcher_request::MatcherRequest;

use crate::models::{event::Event, order::Order, position::Position, user::User};

/// Initializes the in-memory exchange data from the database.
async fn bootstrap_exchange(db: &SqlitePool) -> Exchange {
    let next_order_id = Order::get_next_order_id(db).await;

    let mut balances: HashMap<UserId, Balance> = HashMap::new();
    for user in User::get_with_nonzero_balances(db).await.unwrap() {
        balances.insert(user.id, user.balance);
    }

    let mut events: Vec<EventId> = Vec::new();
    for event in Event::get_active(db).await.unwrap() {
        events.push(event.id);
    }

    let mut positions: HashMap<(UserId, EventId), i32> = HashMap::new();
    for position in Position::get_non_zero(db).await.unwrap() {
        positions.insert((position.user_id, position.event_id), position.position);
    }

    let mut orders: Vec<(UserId, EventId, lobster::Order)> = Vec::new();
    for order_record in Order::get_open_orders(db).await.unwrap() {
        let order = lobster::Order::new(
            order_record.id,
            order_record.quantity,
            order_record.price,
            Side::new(order_record.is_buy),
        );
        orders.push((order_record.user_id, order_record.event_id, order));
    }

    let engine = lobster::Exchange::from_state(
        next_order_id,
        &balances,
        &positions,
        orders.as_slice(),
        events.as_slice(),
    );

    engine
}

pub fn start_matcher_service(
    db: SqlitePool,
    mut recv: mpsc::Receiver<MatcherRequest>,
    market_data: broadcast::Sender<BookUpdate>,
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
                    MatcherRequest::AddEvent { event_id } => {
                        let event = exchange.add_event(timestamp, event_id).unwrap();
                        market_data.send(event).expect("Receiver dropped");
                    }
                    MatcherRequest::Deposit { user, amount } => {
                        exchange.deposit(user, amount);
                    }
                    MatcherRequest::Resolve {
                        event_id,
                        price,
                        response,
                    } => {
                        let event = exchange.resolve(timestamp, event_id, price);
                        response.send(event).expect("Receiver dropped");
                    }
                }
            }
        }
    });
}
