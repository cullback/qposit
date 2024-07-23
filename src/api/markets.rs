use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use lobster::MarketId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::services::matcher_request::MatcherRequest;
use crate::api::feed::BookUpdate;
use crate::app_state::AppState;

use super::api_error::ApiError;
use super::auth::BasicAuthExtractor;

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarketPatchPayload {
    /// If set, resolves the market to the given price.
    #[schema(minimum = 0, maximum = 10000)]
    outcome: Option<u16>,
}

/// Modify an market.
#[utoipa::path(
    patch,
    path = "/api/v1/markets/:id",
    request_body = MarketPatchPayload,
    responses(
        (status = 200, description = "Market modified successfully", body = BookUpdate)
    )
)]
pub async fn patch(
    BasicAuthExtractor(user): BasicAuthExtractor,
    State(state): State<AppState>,
    Path(market_id): Path<MarketId>,
    Json(payload): Json<MarketPatchPayload>,
) -> impl IntoResponse {
    if user.username != "admin" {
        // TODO
        return StatusCode::FORBIDDEN.into_response();
    }

    if let Some(price) = payload.outcome {
        let (cmd, recv) = MatcherRequest::resolve(market_id, price);
        state.cmd_send.send(cmd).await.unwrap();
        let response = recv
            .await
            .unwrap()
            .map_err(|err| ApiError::MatcherRequest(err));
        return match response {
            Ok(market) => Json(BookUpdate::from(market)).into_response(),
            Err(err) => err.into_response(),
        };
    }

    return StatusCode::OK.into_response();
}
