use std::collections::HashMap;

use lobster::{Balance, Side, UserId};
use lobster::{MarketUpdate, Exchange, MarketId};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

use crate::app_state::current_time_micros;

use super::matcher_request::MatcherRequest;

use crate::models::{market::Market, order::Order, position::Position, user::User};

/// Initializes the in-memory exchange data from the database.
async fn bootstrap_exchange(db: &SqlitePool) -> Exchange {
    let next_order_id = Order::get_next_order_id(db).await;

    let mut balances: HashMap<UserId, Balance> = HashMap::new();
    for user in User::get_with_nonzero_balances(db).await.unwrap() {
        balances.insert(user.id, user.balance);
    }

    let mut markets: Vec<MarketId> = Vec::new();
    for market in Market::get_active(db).await.unwrap() {
        markets.push(market.id);
    }

    let mut positions: HashMap<(UserId, MarketId), i32> = HashMap::new();
    for position in Position::get_non_zero(db).await.unwrap() {
        positions.insert((position.user_id, position.market_id), position.position);
    }

    let mut orders: Vec<(UserId, MarketId, lobster::Order)> = Vec::new();
    for order_record in Order::get_open_orders(db).await.unwrap() {
        let order = lobster::Order::new(
            order_record.id,
            order_record.quantity,
            order_record.price,
            Side::new(order_record.is_buy),
        );
        orders.push((order_record.user_id, order_record.market_id, order));
    }

    let engine = lobster::Exchange::from_state(
        next_order_id,
        &balances,
        &positions,
        orders.as_slice(),
        markets.as_slice(),
    );

    engine
}

pub fn start_matcher_service(
    db: SqlitePool,
    mut recv: mpsc::Receiver<MatcherRequest>,
    market_data: broadcast::Sender<MarketUpdate>,
) {
    tokio::spawn({
        async move {
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
                        info!("REQUEST time={timestamp} user={user} post order={order:?}");
                        let res = exchange.submit_order(timestamp, user, order);
                        if let Ok(market) = res.clone() {
                            market_data.send(market).expect("Receiver dropped");
                        }
                        response.send(res).expect("Receiver dropped");
                    }
                    MatcherRequest::CancelOrder {
                        user,
                        order,
                        response,
                    } => {
                        info!("REQUEST time={timestamp} user={user} delete order={order:?}");
                        let res = exchange.cancel_order(timestamp, user, order);
                        if let Ok(market) = res.clone() {
                            market_data.send(market).expect("Receiver dropped");
                        }
                        response.send(res).expect("Receiver dropped");
                    }
                    MatcherRequest::AddMarket { market_id } => {
                        info!("REQUEST time={timestamp} add market={market_id:?}");
                        let market = exchange.add_event(timestamp, market_id).unwrap();
                        market_data.send(market).expect("Receiver dropped");
                    }
                    MatcherRequest::Deposit { user, amount } => {
                        exchange.deposit(user, amount);
                    }
                    MatcherRequest::Resolve {
                        market_id,
                        price,
                        response,
                    } => {
                        info!("REQUEST time={timestamp} resolve={market_id:?} to price={price}");
                        let market = exchange.resolve(timestamp, market_id, price);
                        if let Ok(market) = market.clone() {
                            market_data.send(market).expect("Receiver dropped");
                        }
                        response.send(market).expect("Receiver dropped");
                    }
                }
            }
        }
    });
}
