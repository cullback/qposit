use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use lobster::EventId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::actors::matcher_request::MatcherRequest;
use crate::api::feed::BookUpdate;
use crate::app_state::AppState;

use super::api_error::ApiError;
use super::auth::BasicAuthExtractor;

#[derive(Debug, Deserialize, ToSchema)]
pub struct EventPatchPayload {
    /// If set, resolves the event to the given price.
    #[schema(minimum = 0, maximum = 10000)]
    price: Option<u16>,
}

/// Modify an event.
#[utoipa::path(
    patch,
    path = "/api/v1/events/:id",
    request_body = EventPatchPayload,
    responses(
        (status = 200, description = "Event modified successfully", body = BookUpdate)
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
            Ok(event) => Json(BookUpdate::from(event)).into_response(),
            Err(err) => err.into_response(),
        };
    }

    return StatusCode::OK.into_response();
}
