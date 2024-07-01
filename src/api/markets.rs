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
    authentication::BasicAuthExtractor,
    models::{event::Event, market::Market},
};

#[derive(Debug, Serialize, ToSchema)]
pub struct MarketResponse {
    #[serde(flatten)]
    pub market: Market,
    pub events: Vec<Event>,
}

#[utoipa::path(
    get,
    path = "/api/v1/markets",
    responses(
        (status = 200, description = "Market successfully created", body = [MarketResponse])
    )
)]
pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let markets = Market::get_active_markets(&state.pool).await.unwrap();
    let mut resp = vec![];
    for market in markets {
        let events = Event::get_all_for_market(&state.pool, market.id)
            .await
            .unwrap();

        let market = MarketResponse { market, events };
        resp.push(market);
    }
    Json(resp).into_response()
}

#[derive(Deserialize, ToSchema, Serialize)]
pub struct MarketPost {
    /// The title of the market.
    title: String,
    /// The description of the market.
    description: String,
    /// The time at which the market was created.
    created_at: i64,
    /// The time at which the market will expire.
    expires_at: i64,
    /// The titles for the events.
    events: Vec<String>,
}

/// Posts a new market to the exchange.
/// Creates the associated events as well.
#[utoipa::path(
    post,
    path = "/api/v1/markets",
    request_body = MarketPost,
    responses(
        (status = 200, description = "Market successfully created", body = MarketResponse),
    )
)]
pub async fn post(
    BasicAuthExtractor(user): BasicAuthExtractor,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(market): Json<MarketPost>,
) -> impl IntoResponse {
    // TODO: make this a transaction
    if user.username != "admin" {
        // TODO
        return StatusCode::FORBIDDEN.into_response();
    }

    let record = Market {
        id: 0,
        slug: slug.clone(),
        title: market.title,
        description: market.description,
        status: "active".to_owned(),
        created_at: market.created_at,
        expires_at: market.expires_at,
    };

    let market_id = match record.insert(&state.pool).await {
        Ok(row_id) => row_id,
        Err(sqlx::Error::Database(x)) if x.is_unique_violation() => {
            return Json(json!({"error": "Market already exists"})).into_response();
        }
        Err(_) => {
            return Json(json!({"error": "internal server error"})).into_response();
        }
    };

    for event in market.events {
        let event_id = Event::new(&state.pool, market_id, event).await.unwrap();
        let req = MatcherRequest::AddEvent { event_id };
        state.cmd_send.send(req).await.unwrap();
    }

    let market = Market::get_by_slug(&state.pool, &slug).await.unwrap();

    let events = Event::get_all_for_market(&state.pool, market.id)
        .await
        .unwrap();

    let market = MarketResponse { market, events };

    return (StatusCode::CREATED, Json(market)).into_response();
}
