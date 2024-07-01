use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Executor, Sqlite};
use sqlx::{QueryBuilder, SqliteExecutor};
use utoipa::{IntoParams, ToSchema};

/// A trade.
#[derive(Debug, Serialize, ToSchema, FromRow)]
pub struct Trade {
    /// The ID of the trade.
    pub id: i64,
    /// The timestamp of when the trade was created.
    pub created_at: i64,
    pub tick: u32,
    /// The event ID.
    pub event_id: u32,
    /// The taker's user ID.
    pub taker_id: u32,
    /// The maker's user ID.
    pub maker_id: u32,
    /// The taker's order ID.
    pub taker_oid: i64,
    /// The maker's order ID.
    pub maker_oid: i64,
    /// The quantity of the trade.
    pub quantity: u32,
    /// The price of the trade.
    pub price: u16,
    /// True if the taker is buying.
    pub is_buy: bool,
}

const fn default_limit() -> u32 {
    100
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TradeParams {
    pub event_id: Option<u32>,
    pub user_id: Option<u32>,
    pub before: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

impl Trade {
    pub async fn insert<E>(&self, db: &mut E) -> Result<i64, sqlx::Error>
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        sqlx::query!(
        "
        INSERT INTO trade (created_at, tick, event_id, taker_id, maker_id, taker_oid, maker_oid, quantity, price, is_buy)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        self.created_at,
        self.tick,
        self.event_id,
        self.taker_id,
        self.maker_id,
        self.taker_oid,
        self.maker_oid,
        self.quantity,
        self.price,
        self.is_buy)
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid())
    }

    pub async fn get<'e>(
        db: impl SqliteExecutor<'e>,
        params: TradeParams,
    ) -> Result<Vec<Trade>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT * from trade WHERE 1=1");

        if let Some(event_id) = params.event_id {
            query.push(" AND event_id = ");
            query.push_bind(event_id);
        }
        if let Some(user_id) = params.user_id {
            query.push(" AND (taker_id = ");
            query.push_bind(user_id);
            query.push(" OR maker_id = ");
            query.push_bind(user_id);
            query.push(")");
        }
        if let Some(after) = params.before {
            query.push(" AND created_at < ");
            query.push_bind(after);
        }
        query.push(" ORDER BY created_at DESC LIMIT ");
        query.push_bind(params.limit);

        query.build_query_as::<Trade>().fetch_all(db).await
    }
}
