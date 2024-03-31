use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::{
    app_state::AppState,
    auth::BasicAuthExtractor,
    models::{self, book::Book},
};

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
        (status = 200, description = "Market successfully created", body = [OrderRequest])
    )
)]
pub async fn post(
    State(state): State<AppState>,
    BasicAuthExtractor(user): BasicAuthExtractor,
    Path(slug): Path<String>,
    Extension(db): Extension<SqlitePool>,
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

    let market_id = match record.insert(&db).await {
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
            status: "active".to_owned(),
            value: None,
            last_trade_price: None,
        };
        let book_id = book.insert(&db).await.unwrap();
    }

    StatusCode::CREATED.into_response()
}
