use super::templates::home_page;
use crate::app_state::AppState;
use crate::models::market::Market;
use crate::{auth::SessionExtractor, models::book::Book};
use axum::extract::State;
use axum::response::IntoResponse;

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let markets = Market::get_active_markets(&state.db).await.unwrap();

    let mut blah = Vec::new();
    for market in markets {
        let books = Book::get_all_for_market(&state.db, market.id)
            .await
            .unwrap();
        blah.push((market, books));
    }

    match user {
        Some(user) => home_page::HomePage::new(user.username, blah),
        None => home_page::HomePage::new(String::new(), blah),
    }
}
