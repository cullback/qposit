use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    actors::matcher_request::MatcherRequest,
    app_state::AppState,
    authentication::BasicAuthExtractor,
    models::{self, event::Event},
};

#[utoipa::path(
    get,
    path = "/markets",
    responses(
        (status = 200, description = "Market successfully created")
    )
)]
pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let markets = models::market::Market::get_active_markets(&state.pool)
        .await
        .unwrap();
    let mut resp = vec![];
    for market in markets {
        let events = Event::get_all_for_market(&state.pool, market.id)
            .await
            .unwrap();
        resp.push(json!({
            "slug": market.slug,
            "title": market.title,
            "description": market.description,
            "created_at": market.created_at,
            "expires_at": market.expires_at,
            "events": events.iter().map(|b| json!({
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
    // TODO: make this a transaction
    if user.username != "admin" {
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

    let blah = models::market::Market::get_by_slug(&state.pool, &slug)
        .await
        .unwrap()
        .unwrap();

    return (StatusCode::CREATED, Json(blah)).into_response();
}
