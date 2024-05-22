use askama_axum::IntoResponse;
use axum::{extract::State, http::StatusCode, Json};
use lobster::UserId;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    actors::matcher_request::MatcherRequest, app_state::AppState,
    authentication::BasicAuthExtractor, models::user::User,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DepositPayload {
    pub user_id: UserId,
    pub amount: i64,
}

/// Submit order
///
/// Submit an order to the matching engine.
#[utoipa::path(
    post,
    path = "/deposit",
    security(
        ("basic_auth" = [])
    )
)]
pub async fn deposit(
    State(state): State<AppState>,
    BasicAuthExtractor(user): BasicAuthExtractor,
    Json(payload): Json<DepositPayload>,
) -> impl IntoResponse {
    if user.username != "admin" {
        return Json(json!({"error": "not authorized"})).into_response();
    }

    match User::deposit(&state.db, payload.user_id, payload.amount)
        .await
        .map(|x| x == 1)
    {
        Ok(true) => {} // success
        Ok(false) => return Json(json!({"error": "user not found"})).into_response(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{}", e)})),
            )
                .into_response()
        }
    };
    let req = MatcherRequest::deposit(payload.user_id, payload.amount);
    state.cmd_send.send(req).await.expect("Receiver dropped");

    Json(json!({"success": "hi"})).into_response()
}
