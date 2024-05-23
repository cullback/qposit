use super::templates::{open_orders, positions};
use crate::{app_state::AppState, authentication::SessionExtractor};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
};

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Component<'a> {
    username: &'a str,
    balance: f32,
    positions: positions::Positions,
    open_orders: open_orders::OpenOrders,
}

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(user) = user else {
        return Redirect::to("/").into_response();
    };

    let page = Component {
        username: &user.username,
        balance: user.balance as f32,
        positions: positions::Positions::build(&state.pool, user.id).await,
        open_orders: open_orders::OpenOrders::build(&state.pool, user.id).await,
    }
    .render()
    .unwrap();
    Html(page).into_response()
}
