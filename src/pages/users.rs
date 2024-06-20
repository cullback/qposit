use super::templates::{open_orders, positions, profile};
use crate::{app_state::AppState, authentication::SessionExtractor, models};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user = models::user::User::get_by_username(&state.pool, &username)
        .await
        .unwrap();

    let user_id = user.id;

    profile::Profile::new(
        user,
        positions::Positions::build(&state.pool, user_id).await,
        open_orders::OpenOrders::build(&state.pool, user_id).await,
    )
    .into_response()
}
