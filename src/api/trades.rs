use axum::response::IntoResponse;
use axum::Json;
use axum::{extract::Query, http::StatusCode, response::Response, Extension};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, SqlitePool};
use tracing::{error, info};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, FromRow)]
struct Trade {
    pub id: i64,
    pub created_at: i64,
    pub tick: u32,
    pub book_id: u32,
    pub taker_id: u32,
    pub maker_id: u32,
    pub taker_oid: i64,
    pub maker_oid: i64,
    pub quantity: u32,
    pub price: u16,
    pub is_buy: bool,
}

fn default_limit() -> u32 {
    100
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TradeParams {
    pub book_id: Option<u32>,
    pub user_id: Option<u32>,
    pub before: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Gets recent trades.
///
/// Gets the most recent trades after the specified timestamp.
/// If no timestamp is specified, then it will show the most recent
/// trades; otherwise, it will show the most recent trades that occurred after
/// that timestamp.
#[utoipa::path(
    get,
    path = "/trades",
    responses(
        (status = 200, description = "Success")
    )
)]
pub async fn get(Extension(db): Extension<SqlitePool>, params: Query<TradeParams>) -> Response {
    info!(?params);
    let mut query = QueryBuilder::new("SELECT * from trade WHERE 1 = 1");

    if let Some(book_id) = params.book_id {
        query.push(" AND book_id = ");
        query.push_bind(book_id);
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

    let trades = match query.build_query_as::<Trade>().fetch_all(&db).await {
        Ok(trades) => trades,
        Err(err) => {
            error!(?err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(trades).into_response()
}
