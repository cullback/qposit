//! The writer actor subscribes to the market data feed and records
//! all events to the database.
//! This could be split into a separate microservice, or be duplicated
//! for redundancy.
//! Gets to do less work than the matching engine because all feed events
//! are validated.
use lobster::{
    trade_cost, Action, Balance, BookEvent, BookId, Order, OrderBook, Side, Tick, Timestamp,
    UserId, RESOLVE_PRICE,
};
use lobster::{OrderId, Price, Quantity};
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
    /// Taker side
    side: Side,
}

#[derive(Debug)]
struct OrderOwner {
    user_id: UserId,
    book_id: BookId,
}

struct State {
    db: SqlitePool,
    books: HashMap<BookId, OrderBook>,
    positions: HashMap<(UserId, BookId), i32>,
    order_owner: HashMap<OrderId, OrderOwner>,
}

impl State {
    pub async fn new(db: SqlitePool) -> Self {
        let orders = models::order::Order::get_open_orders(&db).await.unwrap();
        let books = bootstrap_books(&db, orders.as_slice()).await;
        let order_owner = orders
            .iter()
            .map(|order| {
                (
                    order.id,
                    OrderOwner {
                        user_id: order.user_id,
                        book_id: order.book_id,
                    },
                )
            })
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

    async fn on_event(&mut self, event: BookEvent) {
        info!(?event);

        let mut tx = self.db.begin().await.unwrap();
        match event.action {
            Action::Add(order) => {
                self.on_add(
                    &mut *tx, event.time, event.tick, event.user, event.book, order,
                )
                .await
            }
            Action::Remove { id } => self.on_remove(&mut *tx, event.book, id).await,
            Action::Resolve { price } => self.on_resolve(&mut *tx, event.book, price).await,
            Action::AddBook { .. } => {}
        }
        tx.commit().await.unwrap();
    }

    /// This logic is mostly copy-pasted from the matching engine.
    async fn on_trade<E>(&mut self, executor: &mut E, trade: Trade)
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        let position = self
            .positions
            .entry((trade.taker_id, trade.book_id))
            .or_default();
        let taker_cost = trade_cost(*position, trade.quantity, trade.price, trade.side);

        let signed_quantity = match trade.side {
            Side::Buy => trade.quantity as i32,
            Side::Sell => -(trade.quantity as i32),
        };

        *position += signed_quantity;

        let position = self
            .positions
            .entry((trade.maker_id, trade.book_id))
            .or_default();
        let maker_cost = trade_cost(*position, trade.quantity, trade.price, !trade.side);
        *position -= signed_quantity;

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
                taker_cost,
                trade.taker_id,
                // update maker balance params
                maker_cost,
                trade.maker_id,
                // update taker position params
                trade.taker_id,
                trade.book_id,
                signed_quantity,
                signed_quantity,
                // update maker position params
                trade.maker_id,
                trade.book_id,
                signed_quantity,
                signed_quantity,
            )
            .execute(&mut *executor)
            .await
            .unwrap();
        }

        // this is a separate query that runs regardless of self-match or not
        let is_buy = trade.side.is_buy();
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
            is_buy,
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

    async fn on_add<E>(
        &mut self,
        transaction: &mut E,
        time: Timestamp,
        tick: Tick,
        user_id: UserId,
        book_id: BookId,
        order: Order,
    ) where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        models::order::Order::new(&mut *transaction, time, book_id, user_id, order)
            .await
            .unwrap();

        let book = self.books.get_mut(&book_id).unwrap();

        self.order_owner
            .insert(order.id, OrderOwner { user_id, book_id });
        let fills = book.add(order);
        for fill in fills {
            let trade = Trade {
                timestamp: time,
                tick,
                book_id,
                taker_id: user_id,
                maker_id: self.order_owner[&fill.id].user_id,
                taker_oid: order.id,
                maker_oid: fill.id,
                quantity: fill.quantity,
                price: fill.price,
                side: order.side,
            };
            self.on_trade(&mut *transaction, trade).await;
        }
    }

    async fn on_remove<E>(&mut self, transaction: &mut E, book_id: BookId, id: OrderId)
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        let book = self.books.get_mut(&book_id).unwrap();
        assert!(book.remove(id).is_some());
        self.order_owner.remove(&id);

        models::order::Order::cancel_by_id(transaction, id)
            .await
            .unwrap();
    }

    async fn on_resolve<E>(&mut self, transaction: &mut E, book_id: BookId, price: Price)
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        self.books.remove(&book_id).unwrap();
        self.order_owner.retain(|_, order| order.book_id != book_id);

        models::order::Order::cancel_for_book(transaction, book_id)
            .await
            .unwrap();

        for ((user_id, book_id), position) in self.positions.iter_mut() {
            let change = if *position >= 0 {
                Balance::from(price) * Balance::from(*position)
            } else {
                Balance::from(RESOLVE_PRICE - price) * -Balance::from(*position)
            };

            sqlx::query!(
                "
                DELETE FROM 'position' WHERE book_id = ? and user_id = ?;
                UPDATE user SET balance = balance + ? WHERE id = ?;
                ",
                book_id,
                user_id,
                change,
                user_id,
            )
            .execute(&mut *transaction)
            .await
            .unwrap();
        }

        self.positions.retain(|&(_, book), _| book != book_id);
    }
}

pub fn start_writer_service(db: SqlitePool, mut feed: broadcast::Receiver<BookEvent>) {
    tokio::spawn({
        async move {
            info!("Starting writer service...");
            let mut state = State::new(db).await;
            while let Ok(event) = feed.recv().await {
                state.on_event(event).await;
            }
        }
    });
}
