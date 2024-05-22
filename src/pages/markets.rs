use super::templates::market_page::MarketPage;
use crate::app_state::AppState;
use crate::models::market::Market;
use crate::{authentication::SessionExtractor, models::book::Book};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;
use lobster::BookId;

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
        .unwrap()
        .iter()
        .map(|book| book.id)
        .collect::<Vec<BookId>>();

    match user {
        Some(user) => MarketPage::new(user.username, market, books).into_response(),
        None => MarketPage::new(String::new(), market, books).into_response(),
    }
}
