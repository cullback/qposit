//! The writer actor subscribes to the market data feed and records
//! all events to the database.
//! This could be split into a separate microservice, or be duplicated
//! for redundancy.
//! Gets to do less work than the matching engine because all feed events
//! are validated.
use lobster::{
    Action, Balance, BookEvent, BookId, Order, OrderBook, PortfolioManager, Side, Tick, Timestamp,
    UserId,
};
use lobster::{OrderId, Price, Quantity};
use sqlx::{Executor, Sqlite, SqlitePool};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::models;

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
    pub user_id: UserId,
    pub book_id: BookId,
}

struct State {
    db: SqlitePool,
    books: HashMap<BookId, OrderBook>,
    order_owner: HashMap<OrderId, OrderOwner>,
    manager: PortfolioManager,
}

impl State {
    pub async fn new(db: SqlitePool) -> Self {
        let mut balances: HashMap<UserId, Balance> = HashMap::new();
        for user in models::user::User::get_with_nonzero_balances(&db)
            .await
            .unwrap()
        {
            balances.insert(user.id, user.balance);
        }

        let mut positions: HashMap<(UserId, BookId), i32> = HashMap::new();
        for position in models::position::Position::get_non_zero(&db).await.unwrap() {
            positions.insert((position.user_id, position.book_id), position.position);
        }

        let mut manager = PortfolioManager::new(&balances, &positions);

        let mut books: HashMap<BookId, OrderBook> = HashMap::new();
        for book in models::book::Book::get_active(&db).await.unwrap() {
            books.insert(book.id, OrderBook::default());
        }

        let mut order_owner = HashMap::new();
        for order_record in models::order::Order::get_open_orders(&db).await.unwrap() {
            let order = lobster::Order::new(
                order_record.id,
                order_record.quantity,
                order_record.price,
                Side::new(order_record.is_buy),
            );
            order_owner.insert(
                order_record.id,
                OrderOwner {
                    user_id: order_record.user_id,
                    book_id: order_record.book_id,
                },
            );
            manager.add_resting_order(order_record.user_id, order_record.book_id, order);
            let book = books.get_mut(&order_record.book_id).unwrap();
            assert!(book.add(order).is_empty());
        }

        Self {
            db,
            books,
            order_owner,
            manager,
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
            Action::AddBook {} => {
                self.books.insert(event.book, OrderBook::default());
            }
        }
        tx.commit().await.unwrap();
    }

    /// This logic is mostly copy-pasted from the matching engine.
    async fn on_trade<E>(&mut self, executor: &mut E, trade: Trade)
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        self.manager.on_trade(
            trade.taker_id,
            trade.maker_id,
            trade.book_id,
            trade.quantity,
            trade.price,
            trade.side,
        );

        let taker_balance = self.manager.get_balance(trade.taker_id);
        let maker_balance = self.manager.get_balance(trade.maker_id);
        let taker_available = self.manager.get_available(trade.taker_id);
        let maker_available = self.manager.get_available(trade.maker_id);
        let taker_position = self.manager.get_position(trade.taker_id, trade.book_id);
        let maker_position = self.manager.get_position(trade.maker_id, trade.book_id);

        sqlx::query!(
            "
            UPDATE user SET balance = ?, available = ? WHERE id = ?;
            UPDATE user SET balance = ?, available = ? WHERE id = ?;

            INSERT INTO position (user_id, book_id, position)
            VALUES 
                (?, ?, ?),
                (?, ?, ?)
            ON CONFLICT (user_id, book_id) DO UPDATE SET position = excluded.position;
            ",
            taker_balance,
            taker_available,
            trade.taker_id,
            maker_balance,
            maker_available,
            trade.maker_id,
            // update taker position params
            trade.taker_id,
            trade.book_id,
            taker_position,
            // update maker position params
            trade.maker_id,
            trade.book_id,
            maker_position,
        )
        .execute(&mut *executor)
        .await
        .unwrap();

        // this is a separate query that runs regardless of self-match or not
        let is_buy = trade.side.is_buy();
        sqlx::query!(
            "
            INSERT INTO trade (created_at, tick, book_id, taker_id, maker_id, taker_oid, maker_oid, quantity, price, is_buy)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

            UPDATE 'order' SET
                remaining = remaining - ?,
                status = CASE WHEN remaining - ? = 0 THEN 'filled' ELSE 'open' END
            WHERE id IN (?, ?);
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
            // update orders
            trade.quantity,
            trade.quantity,
            trade.taker_oid,
            trade.maker_oid
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
        mut order: Order,
    ) where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        models::order::Order::new(&mut *transaction, time, book_id, user_id, order)
            .await
            .unwrap();

        self.order_owner
            .insert(order.id, OrderOwner { user_id, book_id });

        let book = self.books.get_mut(&book_id).unwrap();
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
            order.quantity -= fill.quantity;
            if fill.done {
                self.order_owner.remove(&fill.id);
            }
        }
        if order.quantity > 0 {
            self.manager.add_resting_order(user_id, book_id, order);
            self.order_owner
                .insert(order.id, OrderOwner { user_id, book_id });

            let available = self.manager.get_available(user_id);
            sqlx::query!(
                "UPDATE user SET available = ? WHERE id = ?",
                available,
                user_id
            )
            .execute(&mut *transaction)
            .await
            .unwrap();
        }
    }

    async fn on_remove<E>(&mut self, transaction: &mut E, book_id: BookId, id: OrderId)
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        models::order::Order::cancel_by_id(transaction, id)
            .await
            .unwrap();

        let book = self.books.get_mut(&book_id).unwrap();
        let order = book.remove(id).unwrap();

        let owner_info = self.order_owner.remove(&id).unwrap();
        self.manager
            .remove_order(owner_info.user_id, book_id, order);
        let available = self.manager.get_available(owner_info.user_id);

        sqlx::query!(
            "UPDATE user SET available = ? WHERE id = ?",
            available,
            owner_info.user_id
        )
        .execute(&mut *transaction)
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

        models::position::Position::delete_for_book(transaction, book_id)
            .await
            .unwrap();

        for user_id in self.manager.resolve(book_id, price) {
            let balance = self.manager.get_balance(user_id);
            let available = self.manager.get_available(user_id);
            sqlx::query!(
                "UPDATE user SET balance = ?, available = ? WHERE id = ?",
                balance,
                available,
                user_id
            )
            .execute(&mut *transaction)
            .await
            .unwrap();
        }
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
