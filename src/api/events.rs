use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    actors::matcher_request::MatcherRequest,
    app_state::AppState,
    models::{event::Event, market::Market},
};

use super::auth::BasicAuthExtractor;

#[derive(Debug, Serialize, ToSchema)]
pub struct EventResponse {
    #[serde(flatten)]
    pub event: Event,
    pub markets: Vec<Market>,
}

#[utoipa::path(
    get,
    path = "/api/v1/events",
    responses(
        (status = 200, description = "Event7 successfully created", body = [EventResponse])
    )
)]
pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let markets = Event::get_active_events(&state.pool).await.unwrap();
    let mut resp = vec![];
    for event in markets {
        let markets = Market::get_all_for_event(&state.pool, event.id)
            .await
            .unwrap();

        let event = EventResponse { event, markets };
        resp.push(event);
    }
    Json(resp).into_response()
}

#[derive(Deserialize, ToSchema, Serialize)]
pub struct EventPost {
    /// The title of the event.
    title: String,
    /// The description of the event.
    description: String,
    /// The time at which the event was created.
    created_at: i64,
    /// The time at which the event will expire.
    expires_at: i64,
    /// The titles for the markets.
    markets: Vec<String>,
}

/// Creates a new event.
///
/// Creates the associated markets as well.
#[utoipa::path(
    post,
    path = "/api/v1/events",
    request_body = EventPost,
    responses(
        (status = 200, description = "Event successfully created", body = EventResponse),
    )
)]
pub async fn post(
    BasicAuthExtractor(user): BasicAuthExtractor,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(event): Json<EventPost>,
) -> impl IntoResponse {
    // TODO: make this a transaction
    if user.username != "admin" {
        // TODO
        return StatusCode::FORBIDDEN.into_response();
    }

    let record = Event {
        id: 0,
        slug: slug.clone(),
        title: event.title,
        description: event.description,
        created_at: event.created_at,
        expires_at: event.expires_at,
    };

    let event_id = match record.insert(&state.pool).await {
        Ok(row_id) => row_id,
        Err(sqlx::Error::Database(x)) if x.is_unique_violation() => {
            return Json(json!({"error": "Event7 already exists"})).into_response();
        }
        Err(_) => {
            return Json(json!({"error": "internal server error"})).into_response();
        }
    };

    for market in event.markets {
        let market_id = Market::new(&state.pool, event_id, market).await.unwrap();
        let req = MatcherRequest::AddEvent { market_id };
        state.cmd_send.send(req).await.unwrap();
    }

    let event = Event::get_by_slug(&state.pool, &slug).await.unwrap();

    let markets = Market::get_all_for_event(&state.pool, event.id)
        .await
        .unwrap();

    let event = EventResponse { event, markets };

    return (StatusCode::CREATED, Json(event)).into_response();
}
