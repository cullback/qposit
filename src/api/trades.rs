use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use axum::{extract::Query, response::Response};

use crate::app_state::AppState;
use crate::models;

use crate::models::trade::TradeParams;

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
pub async fn get<'a>(State(state): State<AppState>, Query(params): Query<TradeParams>) -> Response {
    let trades = models::trade::Trade::get(&state.pool, params)
        .await
        .unwrap();
    Json(trades).into_response()
}
