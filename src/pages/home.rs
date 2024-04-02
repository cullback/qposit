use super::templates::home;
use crate::models::market::Market;
use crate::{auth::SessionExtractor, models::book::Book};
use axum::{
    response::{Html, IntoResponse},
    Extension,
};
use sqlx::SqlitePool;

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    Extension(db): Extension<SqlitePool>,
) -> impl IntoResponse {
    let markets = Market::get_active_markets(&db).await.unwrap();

    let mut blah = Vec::new();
    for market in markets {
        let books = Book::get_all_for_market(&db, market.id).await.unwrap();
        blah.push((market, books));
    }

    match user {
        Some(user) => Html(home::build(&user.username, blah)).into_response(),
        None => Html(home::build("", blah)).into_response(),
    }
}
