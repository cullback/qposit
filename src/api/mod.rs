use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

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
        feed::get,
        trades::get,
        positions::get,
        events::post,
        markets::patch,
    ),
    components(
        schemas(
            order_request::OrderRequest,
            orders::TimeInForce,
            events::EventPost,
            events::EventResponse,
            markets::MarketPatchPayload,
            feed::BookUpdate,
            feed::Action,
            models::order::Order,
            models::event::Event,
            models::market::Market,
            models::position::Position,
            models::trade::Trade,
        ),
    ),
    modifiers(&SecurityAddon),
    tags(
        (
        name = "QPosit",
        description = "
QPosit API.

# Conventions and definitions

## Server Time

The server time is in Coordinated Universal Time (UTC).
"
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
        .route("/markets/:id", patch(markets::patch))
        .route("/feed", get(feed::get))
        .route("/events", get(events::get))
        .route("/events/:slug", get(events::get_by_slug))
        .route("/events/:slug", post(events::post))
        .route(
            "/orders",
            get(orders::get).post(orders::post).delete(orders::delete),
        )
        .route("/orders/:id", delete(orders::delete_by_id))
        .route("/positions", get(positions::get))
        .route("/trades", get(trades::get));

    Router::new()
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", apiv1)
        .with_state(state)
}
