use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use lobster::BookId;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::actors::matcher_request::MatcherRequest;
use crate::api::feed::BookEvent;
use crate::{app_state::AppState, authentication::BasicAuthExtractor};

#[derive(Debug, Deserialize, ToSchema)]
pub struct BookPatchPayload {
    /// If this is set, we resolve the book
    price: Option<u16>,
}

/// Posts a new market to the exchange.
/// Creates the associated books as well.
#[utoipa::path(
    patch,
    path = "/books/:id",
    responses(
        (status = 200, description = "Book successfully modified", body = [BookPatchPayload])
    )
)]
pub async fn patch(
    BasicAuthExtractor(user): BasicAuthExtractor,
    State(state): State<AppState>,
    Path(book_id): Path<BookId>,
    Json(payload): Json<BookPatchPayload>,
) -> impl IntoResponse {
    if user.username != "admin" {
        // TODO
        return StatusCode::FORBIDDEN.into_response();
    }

    if let Some(price) = payload.price {
        let (cmd, recv) = MatcherRequest::resolve(user.id, book_id, price);
        state.cmd_send.send(cmd).await.unwrap();
        let response = recv.await.unwrap();
        return match response {
            Ok(event) => Json(BookEvent::from(event)).into_response(),
            Err(err) => Json(json!({"error": format!("{err:?}")})).into_response(),
        };
    }

    println!("got patch request slug={book_id}, payload={payload:?}");

    return StatusCode::OK.into_response();
}
