use exchange::{BookId, Tick, Timestamp, UserId};
use orderbook::{OrderId, Price, Quantity};
use sqlx::{prelude::FromRow, Executor, Sqlite, SqlitePool};

#[derive(Debug, FromRow)]
pub struct Trade {
    pub created_at: Timestamp,
    pub tick: Tick,
    pub book_id: BookId,
    pub taker_id: UserId,
    pub maker_id: UserId,
    pub taker_oid: OrderId,
    pub maker_oid: OrderId,
    pub quantity: Quantity,
    pub price: Price,
    pub is_buy: bool,
}

impl Trade {
    pub async fn insert<'c, E>(&self, db: E) -> Result<i64, sqlx::Error>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        sqlx::query!(
            "
            INSERT INTO trade (created_at, tick, book_id, taker_id, maker_id, taker_oid, maker_oid, quantity, price, is_buy)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            ",
            self.created_at,
            self.tick,
            self.book_id,
            self.taker_id,
            self.maker_id,
            self.taker_oid,
            self.maker_oid,
            self.quantity,
            self.price,
            self.is_buy,
        )
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid())
    }

    pub async fn get_trades(db: &SqlitePool) -> Result<Vec<Trade>, sqlx::Error> {
        sqlx::query_as::<_, Trade>(
            r#"
        SELECT * FROM 'trade'
        ORDER BY created_at DESC, tick DESC
        "#,
        )
        .fetch_all(db)
        .await
    }
}
