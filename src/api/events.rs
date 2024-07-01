use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use lobster::EventId;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::actors::matcher_request::MatcherRequest;
use crate::api::feed::BookEvent;
use crate::{app_state::AppState, authentication::BasicAuthExtractor};

use super::api_error::ApiError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct EventPatchPayload {
    /// If this is set, we resolve the book
    price: Option<u16>,
}

/// Modify an event.
#[utoipa::path(
    patch,
    path = "/events/:id",
    responses(
        (status = 200, description = "Book successfully modified", body = [EventPatchPayload])
    )
)]
pub async fn patch(
    BasicAuthExtractor(user): BasicAuthExtractor,
    State(state): State<AppState>,
    Path(event_id): Path<EventId>,
    Json(payload): Json<EventPatchPayload>,
) -> impl IntoResponse {
    if user.username != "admin" {
        // TODO
        return StatusCode::FORBIDDEN.into_response();
    }

    if let Some(price) = payload.price {
        let (cmd, recv) = MatcherRequest::resolve(event_id, price);
        state.cmd_send.send(cmd).await.unwrap();
        let response = recv
            .await
            .unwrap()
            .map_err(|err| ApiError::MatcherRequest(err));
        return match response {
            Ok(event) => Json(BookEvent::from(event)).into_response(),
            Err(err) => err.into_response(),
        };
    }

    return StatusCode::OK.into_response();
}
