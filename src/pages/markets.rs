//! The page for a market.
//!
//! We load an initial snapshot of the page and then the websocket feed continuously updates it.
use super::templates::market::MarketPage;
use crate::actors::book_service::EventData;
use crate::app_state::AppState;
use crate::models;
use crate::models::market::Market;
use crate::{authentication::SessionExtractor, models::event::Event};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Ok(market) = Market::get_by_slug(&state.pool, &slug).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Some(market) = market else {
        return Redirect::to("/404").into_response();
    };

    let events = Event::get_all_for_market(&state.pool, market.id)
        .await
        .unwrap();

    let mut new_things = vec![];
    for book in events {
        let orderbook = models::order::Order::build_orderbook(&state.pool, book.id)
            .await
            .unwrap();
        let book_data = EventData::new2(&book, orderbook);
        new_things.push((book, book_data));
    }

    match user {
        Some(user) => MarketPage::new(user.username, market, new_things).into_response(),
        None => MarketPage::new(String::new(), market, new_things).into_response(),
    }
}
