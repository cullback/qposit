use super::templates::{open_orders, positions, profile};
use crate::{app_state::AppState, authentication::SessionExtractor};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(user) = user else {
        return Redirect::to("/").into_response();
    };

    let user_id = user.id;

    profile::Profile::new(
        user,
        positions::Positions::build(&state.pool, user_id).await,
        open_orders::OpenOrders::build(&state.pool, user_id).await,
    )
    .into_response()
}
