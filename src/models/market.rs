use lobster::MarketId;
use serde::Serialize;
use sqlx::{prelude::FromRow, Executor, Sqlite, SqlitePool};
use utoipa::ToSchema;

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct Market {
    pub id: u32,
    pub event_id: i64,
    pub title: String,
    pub outcome: Option<u16>,
    pub last_trade_price: Option<u16>,
    pub best_bid_price: Option<u16>,
    pub best_ask_price: Option<u16>,
    pub volume: i64,
}

impl Market {
    pub async fn new<'c, E: Executor<'c, Database = Sqlite>>(
        db: E,
        event_id: i64,
        title: String,
    ) -> Result<MarketId, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO market (event_id, title)
            VALUES (?, ?)",
            event_id,
            title,
        )
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid() as MarketId)
    }

    pub async fn get_all_for_event(db: &SqlitePool, event: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "
            SELECT
                market.id,
                market.event_id,
                market.title,
                market.outcome,
                (
                    SELECT trade.price
                    FROM trade
                    WHERE trade.market_id = market.id
                    ORDER BY trade.created_at DESC, trade.tick DESC
                    LIMIT 1
                ) AS last_trade_price,
                (
                    SELECT MAX(price)
                    FROM 'order'
                    WHERE 'order'.market_id = market.id AND 'order'.is_buy = 1 AND 'order'.status = 'open'
                ) AS best_bid_price,
                (
                    SELECT MIN(price)
                    FROM 'order'
                    WHERE 'order'.market_id = market.id AND 'order'.is_buy = 0 AND 'order'.status = 'open'
                ) AS best_ask_price,
                (
                    SELECT SUM(quantity * price) FROM trade WHERE market.id = trade.market_id
                ) AS volume
            FROM market
            WHERE market.event_id = ?;
            ",
        )
        .bind(event)
        .fetch_all(db)
        .await
    }

    pub async fn get_active(db: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "
            SELECT
                market.id,
                market.event_id,
                market.title,
                market.outcome,
                (
                    SELECT trade.price
                    FROM trade
                    WHERE trade.market_id = market.id
                    ORDER BY trade.created_at DESC, trade.tick DESC
                    LIMIT 1
                ) as last_trade_price,
                (
                    SELECT MAX(price)
                    FROM 'order'
                    WHERE 'order'.market_id = market.id AND 'order'.is_buy = 1 AND 'order'.status = 'open'
                ) AS best_bid_price,
                (
                    SELECT MIN(price)
                    FROM 'order'
                    WHERE 'order'.market_id = market.id AND 'order'.is_buy = 0 AND 'order'.status = 'open'
                ) AS best_ask_price,
                (
                    SELECT SUM(quantity * price) FROM trade WHERE market.id = trade.market_id
                ) AS volume
            FROM market
            ",
        )
        .fetch_all(db)
        .await
    }
}
