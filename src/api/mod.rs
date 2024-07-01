use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::{app_state::AppState, models};

mod api_error;
mod auth;
mod events;
mod feed;
mod markets;
mod order_request;
mod orders;
mod positions;
mod trades;
mod user;

use utoipa::{
    openapi::{
        self,
        security::{HttpAuthScheme, SecurityScheme},
    },
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        orders::get,
        orders::post,
        orders::delete,
        orders::delete_by_id,
        markets::post,
        feed::get,
        events::patch,
        trades::get,
        positions::get,
    ),
    components(
        schemas(
            order_request::OrderRequest,
            orders::TimeInForce,
            markets::MarketPost,
            markets::MarketResponse,
            trades::Trade,
            events::EventPatchPayload,
            feed::BookUpdate,
            feed::Action,
            models::order::Order,
            models::market::Market,
            models::event::Event,
            models::position::Position,
            trades::Trade,
        ),
    ),
    modifiers(&SecurityAddon),
    tags(
        (
        name = "QPosit",
        description = "QPosit provides a number of Application Programming Interfaces (APIs) through HTTP and Websockets (WS)."
        )
    ),
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "basic_auth",
                SecurityScheme::Http(openapi::security::Http::new(HttpAuthScheme::Basic)),
            )
        }
    }
}

pub fn router(state: AppState) -> Router {
    let apiv1 = Router::new()
        .route("/deposit/:id", post(user::deposit))
        .route("/events/:id", patch(events::patch))
        .route("/feed", get(feed::get))
        .route("/markets", get(markets::get))
        .route("/markets/:slug", post(markets::post))
        .route(
            "/orders",
            get(orders::get).post(orders::post).delete(orders::delete),
        )
        .route("/orders/:id", delete(orders::delete_by_id))
        .route("/positions", get(positions::get))
        .route("/trades", get(trades::get));

    Router::new()
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .nest("/api/v1", apiv1)
        .with_state(state)
}
