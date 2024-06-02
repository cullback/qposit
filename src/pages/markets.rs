//! The page for a market.
//!
//! We load an initial snapshot of the page and then the websocket feed continuously updates it.
use super::templates::market::MarketPage;
use super::OrderBook;
use crate::actors::book_service::BookData;
use crate::app_state::AppState;
use crate::models::market::Market;
use crate::{authentication::SessionExtractor, models::book::Book};
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

    let books = Book::get_all_for_market(&state.pool, market.id)
        .await
        .unwrap();

    let mut new_things = vec![];
    for book in books {
        let book_data = BookData::new(&state.pool, book.id, book.last_trade_price).await;
        new_things.push((book, book_data));
    }

    match user {
        Some(user) => MarketPage::new(user.username, market, new_things).into_response(),
        None => MarketPage::new(String::new(), market, new_things).into_response(),
    }
}
