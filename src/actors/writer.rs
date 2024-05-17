//! The writer actor subscribes to the market data feed and records
//! all events to the database.
//! This could be split into a separate microservice, or be duplicated
//! for redundancy.
use exchange::{buyer_cost, seller_cost, Action, BookEvent, BookId, Tick, Timestamp, UserId};
use orderbook::Book;
use orderbook::{OrderId, Price, Quantity};
use sqlx::{Executor, Sqlite, SqlitePool};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::models;

use super::bootstrap_books;

#[derive(Debug)]
struct Trade {
    timestamp: Timestamp,
    tick: Tick,
    book_id: BookId,
    taker_id: UserId,
    maker_id: UserId,
    taker_oid: OrderId,
    maker_oid: OrderId,
    quantity: Quantity,
    price: Price,
    is_buy: bool,
}

struct State {
    db: SqlitePool,
    books: HashMap<BookId, orderbook::DefaultBook>,
    positions: HashMap<(UserId, BookId), i32>,
    order_owner: HashMap<OrderId, UserId>,
}

impl State {
    pub async fn new(db: SqlitePool) -> Self {
        let orders = models::order::Order::get_open_orders(&db).await.unwrap();
        let books = bootstrap_books(&db, orders.as_slice()).await;
        let order_owner = orders
            .iter()
            .map(|order| (order.id, order.user_id))
            .collect();

        let mut positions = HashMap::new();
        for position in models::position::Position::get_non_zero(&db).await.unwrap() {
            positions.insert((position.user_id, position.book_id), position.position);
        }

        Self {
            db,
            books,
            positions,
            order_owner,
        }
    }

    async fn on_trade<E>(&self, executor: &mut E, trade: Trade)
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        // https://stackoverflow.com/questions/76394665/how-to-pass-sqlx-connection-a-mut-trait-as-a-fn-parameter-in-rust
        let (user_a, user_b) = if trade.is_buy {
            (trade.taker_id, trade.maker_id)
        } else {
            (trade.maker_id, trade.taker_id)
        };

        let pos_a = *self.positions.get(&(user_a, trade.book_id)).unwrap_or(&0);
        let pos_b = *self.positions.get(&(user_b, trade.book_id)).unwrap_or(&0);

        let buyer_cost = buyer_cost(pos_a, trade.quantity, trade.price);
        let seller_cost = seller_cost(pos_b, trade.quantity, trade.price);

        if trade.taker_id == trade.maker_id {
            warn!(?trade, "self trade, not updating balance or position");
        } else {
            sqlx::query!(
                "
                UPDATE user SET balance = balance - ? WHERE id = ?;
                UPDATE user SET balance = balance - ? WHERE id = ?;
    
                INSERT INTO position (user_id, book_id, position)
                VALUES (?, ?, ?) ON CONFLICT (user_id, book_id) DO UPDATE SET position = position + ?;
    
                INSERT INTO position (user_id, book_id, position)
                VALUES (?, ?, -?) ON CONFLICT (user_id, book_id) DO UPDATE SET position = position - ?;
                ",
                // update taker balance params
                buyer_cost,
                trade.taker_id,
                // update maker balance params
                seller_cost,
                trade.maker_id,
                // update taker position params
                user_a,
                trade.book_id,
                trade.quantity,
                trade.quantity,
                // update maker position params
                user_b,
                trade.book_id,
                trade.quantity,
                trade.quantity,
            )
            .execute(&mut *executor)
            .await
            .unwrap();
        }

        sqlx::query!(
            "
            INSERT INTO trade (created_at, tick, book_id, taker_id, maker_id, taker_oid, maker_oid, quantity, price, is_buy)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

            UPDATE 'order'
            SET remaining = remaining - ?,
            status = CASE WHEN remaining - ? = 0 THEN 'filled' ELSE 'open' END
            WHERE id = ?;

            UPDATE 'order'
            SET remaining = remaining - ?,
            status = CASE WHEN remaining - ? = 0 THEN 'filled' ELSE 'open' END
            WHERE id = ?;
            ",
            // trade
            trade.timestamp,
            trade.tick,
            trade.book_id,
            trade.taker_id,
            trade.maker_id,
            trade.taker_oid,
            trade.maker_oid,
            trade.quantity,
            trade.price,
            trade.is_buy,
            // update taker order
            trade.quantity,
            trade.quantity,
            trade.taker_oid,
            // update maker order
            trade.quantity,
            trade.quantity,
            trade.maker_oid,
        )
        .execute(&mut *executor)
        .await
        .unwrap();
    }

    async fn on_event(&mut self, event: BookEvent) {
        info!(?event);
        let book = self.books.get_mut(&event.book).unwrap();

        let mut transaction = self.db.begin().await.unwrap();
        match event.action {
            Action::Add(order) => {
                models::order::Order {
                    id: order.id,
                    created_at: event.time,
                    book_id: event.book,
                    user_id: event.user,
                    quantity: order.quantity,
                    remaining: order.quantity,
                    price: order.price,
                    is_buy: order.side.is_buy(),
                    status: "open".to_string(),
                }
                .insert(&mut *transaction)
                .await
                .unwrap();

                self.order_owner.insert(order.id, event.user);
                let fills = book.add(order);

                for fill in fills {
                    let trade = Trade {
                        timestamp: event.time,
                        tick: event.tick,
                        book_id: event.book,
                        taker_id: event.user,
                        maker_id: self.order_owner[&fill.id],
                        taker_oid: order.id,
                        maker_oid: fill.id,
                        quantity: fill.quantity,
                        price: fill.price,
                        is_buy: order.side.is_buy(),
                    };
                    self.on_trade(&mut *transaction, trade).await;
                }
            }
            Action::Remove { id } => {
                assert!(book.remove(id).is_some());
                self.order_owner.remove(&id);

                sqlx::query!("UPDATE 'order' SET status = 'cancelled' WHERE id = ?", id)
                    .execute(&mut *transaction)
                    .await
                    .unwrap();
            }
        }
        transaction.commit().await.unwrap();
    }
}

/// Runs the exchange service
pub async fn run_persistor(db: SqlitePool, mut feed: broadcast::Receiver<BookEvent>) {
    info!("Starting persistor...");
    let mut state = State::new(db).await;

    while let Ok(event) = feed.recv().await {
        state.on_event(event).await;
    }
}
