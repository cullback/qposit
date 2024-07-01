use askama_axum::IntoResponse;
use axum::{extract::State, Json};

use crate::{app_state::AppState, models::position::Position};

use super::auth::BasicAuthExtractor;

/// Get user info.
#[utoipa::path(
    get,
    path = "/api/v1/positions",
    security(
        ("basic_auth" = [])
    )
)]
pub async fn get(
    BasicAuthExtractor(user): BasicAuthExtractor,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let positions = Position::get_for_user(&state.pool, user.id).await.unwrap();

    Json(positions).into_response()
}
