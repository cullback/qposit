use super::auth::SessionExtractor;
use super::templates::home_page::HomePage;
use crate::app_state::AppState;
use crate::models::event::Event;
use crate::models::market::Market;
use axum::extract::State;
use axum::response::IntoResponse;

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let active_markets = Market::get_active_markets(&state.pool).await.unwrap();

    let mut markets = Vec::new();
    for market in active_markets {
        let events = Event::get_all_for_market(&state.pool, market.id)
            .await
            .unwrap();
        markets.push((market, events));
    }

    match user {
        Some(user) => HomePage::new(user.username, markets),
        None => HomePage::new(String::new(), markets),
    }
}
