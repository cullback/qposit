use super::templates::home_page;
use crate::models::market::Market;
use crate::{auth::SessionExtractor, models::book::Book};
use axum::{response::IntoResponse, Extension};
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
        Some(user) => home_page::HomePage::new(user.username, blah),
        None => home_page::HomePage::new(String::new(), blah),
    }
}
