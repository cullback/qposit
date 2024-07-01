use lobster::EventId;
use serde::Serialize;
use sqlx::{prelude::FromRow, Executor, Sqlite, SqlitePool};
use utoipa::ToSchema;

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct Event {
    pub id: u32,
    pub market_id: i64,
    pub title: String,
    pub value: Option<u16>,
    pub last_trade_price: Option<u16>,
    pub best_bid_price: Option<u16>,
    pub best_ask_price: Option<u16>,
    pub volume: i64,
}

impl Event {
    pub async fn new<'c, E: Executor<'c, Database = Sqlite>>(
        db: E,
        market_id: i64,
        title: String,
    ) -> Result<EventId, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO event (market_id, title)
            VALUES (?, ?)",
            market_id,
            title,
        )
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid() as EventId)
    }

    pub async fn get_all_for_market(
        db: &SqlitePool,
        market: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "
            SELECT
                event.id,
                event.market_id,
                event.title,
                event.value,
                (
                    SELECT trade.price
                    FROM trade
                    WHERE trade.event_id = event.id
                    ORDER BY trade.created_at DESC, trade.tick DESC
                    LIMIT 1
                ) AS last_trade_price,
                (
                    SELECT MAX(price)
                    FROM 'order'
                    WHERE 'order'.event_id = event.id AND 'order'.is_buy = 1 AND 'order'.status = 'open'
                ) AS best_bid_price,
                (
                    SELECT MIN(price)
                    FROM 'order'
                    WHERE 'order'.event_id = event.id AND 'order'.is_buy = 0 AND 'order'.status = 'open'
                ) AS best_ask_price,
                (
                    SELECT SUM(quantity * price) FROM trade WHERE event.id = trade.event_id
                ) AS volume
            FROM event
            WHERE event.market_id = ?;
            ",
        )
        .bind(market)
        .fetch_all(db)
        .await
    }

    pub async fn get_active(db: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "
            SELECT
                event.id,
                event.market_id,
                event.title,
                event.value,
                (
                    SELECT trade.price
                    FROM trade
                    WHERE trade.event_id = event.id
                    ORDER BY trade.created_at DESC, trade.tick DESC
                    LIMIT 1
                ) as last_trade_price,
                (
                    SELECT MAX(price)
                    FROM 'order'
                    WHERE 'order'.event_id = event.id AND 'order'.is_buy = 1 AND 'order'.status = 'open'
                ) AS best_bid_price,
                (
                    SELECT MIN(price)
                    FROM 'order'
                    WHERE 'order'.event_id = event.id AND 'order'.is_buy = 0 AND 'order'.status = 'open'
                ) AS best_ask_price,
                (
                    SELECT SUM(quantity * price) FROM trade WHERE event.id = trade.event_id
                ) AS volume
            FROM event
            ",
        )
        .fetch_all(db)
        .await
    }
}
