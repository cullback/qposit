use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use exchange::BookId;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    actors::matcher_request::MatcherRequest,
    app_state::AppState,
    auth::BasicAuthExtractor,
    models::{self, book::Book},
};

#[utoipa::path(
    get,
    path = "/markets",
    responses(
        (status = 200, description = "Market successfully created")
    )
)]
pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let markets = models::market::Market::get_active_markets(&state.db)
        .await
        .unwrap();
    let mut resp = vec![];
    for market in markets {
        let books = Book::get_all_for_market(&state.db, market.id)
            .await
            .unwrap();
        resp.push(json!({
            "title": market.title,
            "description": market.description,
            "created_at": market.created_at,
            "expires_at": market.expires_at,
            "books": books.iter().map(|b| json!({
                "id": b.id.to_string(),
                "title": b.title,
                "value": b.value,
                "last_trade_price": b.last_trade_price,
            })).collect::<Vec<_>>(),
        }));
    }
    Json(resp).into_response()
}

#[derive(Deserialize, ToSchema)]
pub struct Market {
    title: String,
    description: String,
    created_at: i64,
    expires_at: i64,
    books: Vec<String>,
}

/// Posts a new market to the exchange.
/// Creates the associated books as well.
#[utoipa::path(
    post,
    path = "/markets",
    responses(
        (status = 200, description = "Market successfully created", body = [Market])
    )
)]
pub async fn post(
    BasicAuthExtractor(user): BasicAuthExtractor,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(market): Json<Market>,
) -> impl IntoResponse {
    if user.username != "testaccount" {
        // TODO
        return StatusCode::FORBIDDEN.into_response();
    }

    let record = models::market::Market {
        id: 0,
        slug: slug.clone(),
        title: market.title,
        description: market.description,
        status: "active".to_owned(),
        created_at: market.created_at,
        expires_at: market.expires_at,
    };

    let market_id = match record.insert(&state.db).await {
        Ok(row_id) => row_id,
        Err(sqlx::Error::Database(x)) if x.is_unique_violation() => {
            return Json(json!({"error": "Market already exists"})).into_response();
        }
        Err(_) => {
            return Json(json!({"error": "internal server error"})).into_response();
        }
    };

    for book in market.books {
        let book = Book {
            id: 0,
            market_id,
            title: book,
            value: None,
            last_trade_price: None,
        };
        let book_id = book.insert(&state.db).await.unwrap() as BookId;
        let req = MatcherRequest::AddBook { book_id };
        state.cmd_send.send(req).await.unwrap();
    }

    StatusCode::CREATED.into_response()
}
