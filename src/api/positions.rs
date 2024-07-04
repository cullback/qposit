use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    app_state::AppState,
    models::position::{Position, PositionParams},
};

use super::auth::OptionalBasicAuth;

/// Get user info.
#[utoipa::path(
    get,
    path = "/api/v1/positions",
    security(
        ("basic_auth" = [])
    )
)]
pub async fn get(
    OptionalBasicAuth(user): OptionalBasicAuth,
    State(state): State<AppState>,
    Query(params): Query<PositionParams>,
) -> impl IntoResponse {
    let positions = Position::get(&state.pool, params).await.unwrap();
    Json(positions)
}
