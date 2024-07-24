//! The page for a event.
//!
//! We load an initial snapshot of the page and then the websocket feed continuously updates it.
use super::auth::SessionExtractor;
use super::templates::event::EventPage;
use crate::app_state::AppState;
use crate::models;
use crate::models::event::Event;
use crate::models::market::Market;
use crate::services::book_service::MarketData;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let event = match Event::get_by_slug(&state.pool, &slug).await {
        Ok(event) => event,
        Err(sqlx::Error::RowNotFound) => return Redirect::to("/404").into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let markets = Market::get_all_for_event(&state.pool, event.id)
        .await
        .unwrap();

    let mut new_things = vec![];
    for market in markets {
        let orderbook = models::order::Order::build_orderbook(&state.pool, market.id)
            .await
            .unwrap();
        let book_data = MarketData::new(&market, orderbook);
        new_things.push((market, book_data));
    }

    match user {
        Some(user) => EventPage::new(user.username, event, new_things).into_response(),
        None => EventPage::new(String::new(), event, new_things).into_response(),
    }
}
