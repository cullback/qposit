use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use lobster::UserId;
use serde::Deserialize;
use serde_json::json;
use tracing::error;
use utoipa::ToSchema;

use crate::{actors::matcher_request::MatcherRequest, app_state::AppState, models::user::User};

use super::auth::BasicAuthExtractor;

#[derive(Debug, Deserialize, ToSchema)]
pub struct DepositPayload {
    pub amount: i64,
}

/// Deposit.
///
/// Increase a users balance.
#[utoipa::path(
    post,
    path = "/api/v1/deposit/:user_id",
    security(
        ("basic_auth" = [])
    )
)]
pub async fn deposit(
    State(state): State<AppState>,
    BasicAuthExtractor(user): BasicAuthExtractor,
    Path(user_id): Path<UserId>,
    Json(payload): Json<DepositPayload>,
) -> impl IntoResponse {
    // this is a post request because it creates a new entry in transactions table.
    if user.username != "admin" {
        return Json(json!({"error": "not authorized"})).into_response();
    }

    if payload.amount <= 0 {
        return Json(json!({"error": "amount must be positive"})).into_response();
    }

    // deposit to database first
    match User::deposit(&state.pool, user_id, payload.amount).await {
        Ok(_) => {} // success
        Err(sqlx::Error::RowNotFound) => {
            return Json(json!({"error": "user not found"})).into_response()
        }
        Err(e) => {
            error!("Failed to deposit: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let req = MatcherRequest::deposit(user_id, payload.amount);
    state.cmd_send.send(req).await.expect("Receiver dropped");

    let user = User::get_by_id(&state.pool, user_id).await.unwrap();

    Json(json!({"balance": user.balance + payload.amount})).into_response()
}
