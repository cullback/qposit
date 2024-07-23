//! The writer actor subscribes to the event data feed and records
//! all markets to the database.
//! This could be split into a separate microservice, or be duplicated
//! for redundancy.
//! Gets to do less work than the matching engine because all feed markets
//! are validated.
use lobster::{
    Action, Balance, BookUpdate, MarketId, Order, OrderBook, PortfolioManager, Side, Tick,
    Timestamp, UserId,
};
use lobster::{OrderId, Price};
use sqlx::{Executor, Sqlite, SqlitePool};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::models;
use crate::models::trade::Trade;

#[derive(Debug)]
struct OrderOwner {
    pub user_id: UserId,
    pub market_id: MarketId,
}

struct State {
    db: SqlitePool,
    orderbooks: HashMap<MarketId, OrderBook>,
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

        let mut positions: HashMap<(UserId, MarketId), i32> = HashMap::new();
        for position in models::position::Position::get_non_zero(&db).await.unwrap() {
            positions.insert((position.user_id, position.market_id), position.position);
        }

        let mut manager = PortfolioManager::new(&balances, &positions);

        let mut orderbooks: HashMap<MarketId, OrderBook> = HashMap::new();
        for book in models::market::Market::get_active(&db).await.unwrap() {
            orderbooks.insert(book.id, OrderBook::default());
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
                    market_id: order_record.market_id,
                },
            );
            manager.add_resting_order(order_record.user_id, order_record.market_id, order);
            let book = orderbooks.get_mut(&order_record.market_id).unwrap();
            assert!(book.add(order).is_empty());
        }

        Self {
            db,
            orderbooks,
            order_owner,
            manager,
        }
    }

    async fn on_event(&mut self, update: BookUpdate) {
        info!(?update);

        let mut tx = self.db.begin().await.unwrap();
        match update.action {
            Action::Add(order) => {
                self.on_add(
                    &mut *tx,
                    update.time,
                    update.tick,
                    update.user,
                    update.book,
                    order,
                )
                .await
            }
            Action::Remove { id } => self.on_remove(&mut *tx, update.book, id).await,
            Action::Resolve { price } => self.on_resolve(&mut *tx, update.book, price).await,
            Action::AddMarket {} => {
                self.orderbooks.insert(update.book, OrderBook::default());
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
            trade.market_id,
            trade.quantity,
            trade.price,
            Side::new(trade.is_buy),
        );

        let taker_balance = self.manager.get_balance(trade.taker_id);
        let maker_balance = self.manager.get_balance(trade.maker_id);
        let taker_available = self.manager.get_available(trade.taker_id);
        let maker_available = self.manager.get_available(trade.maker_id);
        let taker_position = self.manager.get_position(trade.taker_id, trade.market_id);
        let maker_position = self.manager.get_position(trade.maker_id, trade.market_id);

        sqlx::query!(
            "
            UPDATE user SET balance = ?, available = ? WHERE id = ?;
            UPDATE user SET balance = ?, available = ? WHERE id = ?;

            INSERT INTO position (user_id, market_id, position)
            VALUES 
                (?, ?, ?),
                (?, ?, ?)
            ON CONFLICT (user_id, market_id) DO UPDATE SET position = excluded.position;
            ",
            taker_balance,
            taker_available,
            trade.taker_id,
            maker_balance,
            maker_available,
            trade.maker_id,
            // update taker position params
            trade.taker_id,
            trade.market_id,
            taker_position,
            // update maker position params
            trade.maker_id,
            trade.market_id,
            maker_position,
        )
        .execute(&mut *executor)
        .await
        .unwrap();

        trade.insert(executor).await.unwrap();

        sqlx::query!(
            "
            UPDATE 'order' SET
                remaining = remaining - ?,
                status = CASE WHEN remaining - ? = 0 THEN 'filled' ELSE 'open' END
            WHERE id IN (?, ?);
            ",
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
        market_id: MarketId,
        mut order: Order,
    ) where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        models::order::Order::new(&mut *transaction, time, market_id, user_id, order)
            .await
            .unwrap();

        self.order_owner
            .insert(order.id, OrderOwner { user_id, market_id });

        let book = self.orderbooks.get_mut(&market_id).unwrap();
        let fills = book.add(order);
        for fill in fills {
            let trade = Trade {
                id: 0,
                created_at: time,
                tick,
                market_id,
                taker_id: user_id,
                maker_id: self.order_owner[&fill.id].user_id,
                taker_oid: order.id,
                maker_oid: fill.id,
                quantity: fill.quantity,
                price: fill.price,
                is_buy: order.side.is_buy(),
            };
            self.on_trade(&mut *transaction, trade).await;
            order.quantity -= fill.quantity;
            if fill.done {
                self.order_owner.remove(&fill.id);
            }
        }
        if order.quantity > 0 {
            self.manager.add_resting_order(user_id, market_id, order);
            self.order_owner
                .insert(order.id, OrderOwner { user_id, market_id });

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

    async fn on_remove<E>(&mut self, transaction: &mut E, market_id: MarketId, id: OrderId)
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        models::order::Order::cancel_by_id(transaction, id)
            .await
            .unwrap();

        let book = self.orderbooks.get_mut(&market_id).unwrap();
        let order = book.remove(id).unwrap();

        let owner_info = self.order_owner.remove(&id).unwrap();
        self.manager
            .remove_order(owner_info.user_id, market_id, order);
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

    async fn on_resolve<E>(&mut self, transaction: &mut E, market_id: MarketId, price: Price)
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        models::market::Market::resolve(transaction, market_id, price)
            .await
            .unwrap();

        self.orderbooks.remove(&market_id).unwrap();
        self.order_owner
            .retain(|_, order| order.market_id != market_id);

        models::order::Order::cancel_for_event(transaction, market_id)
            .await
            .unwrap();

        models::position::Position::delete_for_event(transaction, market_id)
            .await
            .unwrap();

        for user_id in self.manager.resolve(market_id, price) {
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

pub fn start_writer_service(db: SqlitePool, mut feed: broadcast::Receiver<BookUpdate>) {
    tokio::spawn({
        async move {
            info!("Starting writer service...");
            let mut state = State::new(db).await;
            while let Ok(market) = feed.recv().await {
                state.on_event(market).await;
            }
        }
    });
}
