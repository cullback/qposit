use axum::{
    response::{Html, IntoResponse},
    Extension,
};
use sqlx::SqlitePool;

use crate::{auth::SessionExtractor, models::book::Book};

use askama::Template;

use crate::models::market::Market;

#[derive(Template)]
#[template(path = "home.html")]
pub struct Component<'a> {
    username: &'a str,
    markets: Vec<(Market, Vec<Book>)>,
}

pub fn build(username: &str, markets: Vec<(Market, Vec<Book>)>) -> String {
    Component { username, markets }.render().unwrap()
}

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
        Some(user) => Html(build(&user.username, blah)).into_response(),
        None => Html(build("", blah)).into_response(),
    }
}
