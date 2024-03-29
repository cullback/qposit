use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use exchange::{BookId, Timestamp};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::{
    app_state::{current_time_micros, AppState},
    auth::BearerExtractor,
    models,
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
    BearerExtractor(user): BearerExtractor,
    Path(slug): Path<String>,
    Extension(db): Extension<SqlitePool>,
    Json(market): Json<Market>,
) -> impl IntoResponse {
    if user.username != "admin" {
        return StatusCode::FORBIDDEN.into_response();
    }

    let timestamp = current_time_micros();

    let record = models::market::Market {
        id: 0,
        slug: slug.clone(),
        title: market.title,
        description: market.description,
        status: "active".to_owned(),
        created_at: timestamp,
        expires_at: 0,
    };
    match record.insert(&db).await {
        Ok(_) => {}
        Err(sqlx::Error::Database(x)) if x.is_unique_violation() => {
            return Json(json!({"error": "Market already exists"})).into_response();
        }
        Err(err) => {
            return Json(json!({"error": format!("{err:?}")})).into_response();
        }
    }

    for book in market.books {
        let row = sqlx::query!(
            "INSERT INTO book (market_id, name, status) VALUES (?, ?, 'active')",
            slug,
            book
        )
        .execute(&db)
        .await
        .unwrap();

        let book_id = row.last_insert_rowid() as BookId;
        // state
        //     .engine
        //     .send(EngineRequest::AddBook { book_id })
        //     .await
        //     .expect("Engine receiver dropped");
    }

    StatusCode::CREATED.into_response()
}
