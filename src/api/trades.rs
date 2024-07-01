use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use axum::{extract::Query, http::StatusCode, response::Response};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::QueryBuilder;
use tracing::error;
use utoipa::{IntoParams, ToSchema};

use crate::app_state::AppState;

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

/// Gets recent trades.
///
/// Gets the most recent trades after the specified timestamp.
/// If no timestamp is specified, then it will show the most recent
/// trades; otherwise, it will show the most recent trades that occurred after
/// that timestamp.
#[utoipa::path(
    get,
    path = "/api/v1/trades",
    params(TradeParams),
    responses(
        (status = 200, description = "Success", body = [Trade])
    )
)]
pub async fn get(State(state): State<AppState>, params: Query<TradeParams>) -> Response {
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

    let trades = match query.build_query_as::<Trade>().fetch_all(&state.pool).await {
        Ok(trades) => trades,
        Err(err) => {
            error!(?err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(trades).into_response()
}
