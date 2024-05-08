use super::orderbook::book_snapshot;
use super::templates::market_page::MarketPage;
use crate::app_state::AppState;
use crate::models::market::Market;
use crate::{auth::SessionExtractor, models::book::Book};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Ok(market) = Market::get_by_slug(&state.db, &slug).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Some(market) = market else {
        return Redirect::to("/404").into_response();
    };

    let books = Book::get_all_for_market(&state.db, market.id)
        .await
        .unwrap();

    let mut orderbooks = Vec::new();
    for book in &books {
        let orderbook = book_snapshot(&state.db, book.id).await;
        orderbooks.push(orderbook);
    }

    match user {
        Some(user) => MarketPage::new(user.username, market, books).into_response(),
        None => MarketPage::new(String::new(), market, books).into_response(),
    }
}
