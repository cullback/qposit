use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::{app_state::AppState, models};

mod api_error;
mod events;
mod feed;
mod markets;
mod order_request;
mod orders;
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

pub struct ApiDoc;

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
        .route(
            "/orders",
            get(orders::get).post(orders::post).delete(orders::delete),
        )
        .route("/orders/:id", delete(orders::delete_by_id))
        .route("/markets/:slug", post(markets::post))
        .route("/markets", get(markets::get))
        .route("/trades", get(trades::get))
        .route("/events/:id", patch(events::patch))
        .route("/deposit/:id", post(user::deposit))
        .route("/feed", get(feed::get));

    Router::new()
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .nest("/api/v1", apiv1)
        .with_state(state)
}
