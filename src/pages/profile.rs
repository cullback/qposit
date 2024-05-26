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

    let page = profile::Profile::new(
        user.username,
        user.balance,
        positions::Positions::build(&state.pool, user.id).await,
        open_orders::OpenOrders::build(&state.pool, user.id).await,
    );

    page.into_response()
}
