use axum::{
    routing::{delete, get, post},
    Router,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::Redoc;
use utoipa_redoc::Servable;
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;

mod feed;
mod markets;
mod order_request;
mod orders;
mod trades;

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
        trades::get,
    ),
    components(
        schemas(order_request::OrderRequest, orders::TimeInForce, orders::OrderResponse, markets::Market, trades::Trade),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "QPosit", description = "QPosit provides a number of Application Programming Interfaces (APIs) through HTTP and Websockets (WS).")
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
        .route("/feed", get(feed::get));

    Router::new()
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .nest("/api/v1", apiv1)
        .with_state(state)
}
