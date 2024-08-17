use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    Json,
};
use lobster::UserId;
use serde::Deserialize;
use serde_json::json;
use tracing::error;
use utoipa::ToSchema;

use crate::{app_state::AppState, models::user::User, services::matcher_request::MatcherRequest};

use super::{api_error::ApiError, auth::OptionalBasicAuth};
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
        return ApiError::Authorization.into_response();
    }

    if payload.amount <= 0 {
        return Json(json!({"error": "amount must be positive"})).into_response();
    }

    let mut user = match User::get_by_id(&state.pool, user_id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return ApiError::UserNotFound.into_response();
        }
        Err(e) => {
            error!("Failed to deposit: {:?}", e);
            return ApiError::InternalServerError.into_response();
        }
    };

    let req = MatcherRequest::deposit(user_id, payload.amount);
    state.cmd_send.send(req).await.unwrap();

    user.balance += payload.amount;
    user.available += payload.amount;

    return Json(user).into_response();
}


pub async fn get(
    State(state): State<AppState>,
    OptionalBasicAuth(_user): OptionalBasicAuth,
    Path(username): Path<String>,
) -> impl IntoResponse {

    let user = match User::get_by_username(&state.pool, &username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return ApiError::UserNotFound.into_response();
        }
        Err(e) => {
            error!("Failed to get user: {:?}", e);
            return ApiError::InternalServerError.into_response();
        }
    };

    return Json(user).into_response();
}