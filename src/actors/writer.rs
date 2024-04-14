//! The writer actor subscribes to the market data feed and records
//! all events to the database.
//! This could be split into a separate microservice, or be duplicated
//! for redundancy.
use exchange::{buyer_cost, seller_cost, Action, BookEvent, BookId, Tick, Timestamp, UserId};
use orderbook::{Book, DefaultBook};
use orderbook::{OrderId, Price, Quantity};
use sqlx::{Executor, Sqlite, SqlitePool};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::models;

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
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            books: HashMap::new(),
            positions: HashMap::new(),
            order_owner: HashMap::new(),
        }
    }

    async fn on_trade<'c, E>(&mut self, executor: E, trade: Trade)
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let (user_a, user_b) = if trade.is_buy {
            (trade.taker_id, trade.maker_id)
        } else {
            (trade.maker_id, trade.taker_id)
        };

        let pos_a = *self.positions.get(&(user_a, trade.book_id)).unwrap_or(&0);
        let pos_b = *self.positions.get(&(user_b, trade.book_id)).unwrap_or(&0);

        let buyer_cost = buyer_cost(pos_a, trade.quantity, trade.price);
        let seller_cost = seller_cost(pos_b, trade.quantity, trade.price);

        sqlx::query!(
            "
            UPDATE user SET balance = balance - ? WHERE id = ?;
            UPDATE user SET balance = balance - ? WHERE id = ?;

            UPDATE position SET position = position + ? WHERE user_id = ? AND book_id = ?;
            UPDATE position SET position = position - ? WHERE user_id = ? AND book_id = ?;

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
            // update taker balance params
            buyer_cost,
            trade.taker_id,
            // update maker balance params
            seller_cost,
            trade.maker_id,
            // update taker position params
            trade.quantity,
            user_a,
            trade.book_id,
            // update maker position params
            trade.quantity,
            user_b,
            trade.book_id,
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
        .execute(executor)
        .await
        .unwrap();
    }

    async fn on_event(&mut self, event: BookEvent) {
        let book = self.books.get_mut(&event.book).unwrap();

        let mut transaction = self.db.begin().await.unwrap();
        match event.action {
            Action::Add {
                id,
                quantity,
                price,
                is_buy,
            } => {
                models::order::Order {
                    id,
                    created_at: event.time,
                    book_id: event.book,
                    user_id: event.user,
                    quantity,
                    remaining: quantity,
                    price,
                    is_buy,
                    status: "open".to_string(),
                }
                .insert(&mut *transaction)
                .await
                .unwrap();

                self.order_owner.insert(id, event.user);
                let fills = if is_buy {
                    book.buy(id, quantity, price)
                } else {
                    book.sell(id, quantity, price)
                };

                for fill in fills {
                    let trade = Trade {
                        timestamp: event.time,
                        tick: event.tick,
                        book_id: event.book,
                        taker_id: event.user,
                        maker_id: self.order_owner[&fill.id],
                        taker_oid: id,
                        maker_oid: fill.id,
                        quantity: fill.quantity,
                        price: fill.price,
                        is_buy,
                    };
                    self.on_trade(&mut *transaction, trade).await;
                }
            }
            Action::Remove { id } => {
                assert!(book.remove(id));

                sqlx::query!("UPDATE 'order' SET status = 'cancelled' WHERE id = ?", id)
                    .execute(&mut *transaction)
                    .await
                    .unwrap();
            }
        }
        transaction.commit().await.unwrap();
    }

    async fn initialize(&mut self) {
        for book in models::book::Book::get_active(&self.db).await.unwrap() {
            self.books.insert(book.id, DefaultBook::default());
        }

        for order in models::order::Order::get_open_orders(&self.db)
            .await
            .unwrap()
        {
            if order.is_buy {
                self.books.get_mut(&order.book_id).unwrap().buy(
                    order.id,
                    order.remaining,
                    order.price,
                );
            } else {
                self.books.get_mut(&order.book_id).unwrap().sell(
                    order.id,
                    order.remaining,
                    order.price,
                );
            }
            self.order_owner.insert(order.id, order.user_id);
        }

        for position in models::position::Position::get_non_zero(&self.db)
            .await
            .unwrap()
        {
            self.positions
                .insert((position.user_id, position.book_id), position.position);
        }
    }
}

/// Runs the exchange service
pub async fn run_persistor(db: SqlitePool, mut feed: broadcast::Receiver<BookEvent>) {
    info!("Starting persistor...");
    let mut state = State::new(db);
    state.initialize().await;

    while let Ok(event) = feed.recv().await {
        state.on_event(event).await;
    }
}
