use super::templates::home_page::HomePage;
use crate::app_state::AppState;
use crate::models::market::Market;
use crate::{authentication::SessionExtractor, models::book::Book};
use axum::extract::State;
use axum::response::IntoResponse;

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let active_markets = Market::get_active_markets(&state.db).await.unwrap();

    let mut markets = Vec::new();
    for market in active_markets {
        let books = Book::get_all_for_market(&state.db, market.id)
            .await
            .unwrap();
        markets.push((market, books));
    }

    match user {
        Some(user) => HomePage {
            username: user.username,
            markets,
        },
        None => HomePage {
            username: String::new(),
            markets,
        },
    }
}
