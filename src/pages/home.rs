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
    let active_events = Event::get_active_events(&state.pool).await.unwrap();

    let mut events = Vec::new();
    for event in active_events {
        let markets = Market::get_all_for_event(&state.pool, event.id)
            .await
            .unwrap();
        events.push((event, markets));
    }

    match user {
        Some(user) => HomePage::new(user.username, events),
        None => HomePage::new(String::new(), events),
    }
}
