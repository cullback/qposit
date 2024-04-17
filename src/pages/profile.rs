use super::templates::{open_orders, positions};
use crate::auth::SessionExtractor;
use askama::Template;
use axum::{
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use sqlx::SqlitePool;

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
    Extension(db): Extension<SqlitePool>,
) -> impl IntoResponse {
    let Some(user) = user else {
        return Redirect::to("/").into_response();
    };

    let page = Component {
        username: &user.username,
        balance: user.balance as f32,
        positions: positions::Positions::build(&db, user.id).await,
        open_orders: open_orders::OpenOrders::build(&db, user.id).await,
    }
    .render()
    .unwrap();
    Html(page).into_response()
}
