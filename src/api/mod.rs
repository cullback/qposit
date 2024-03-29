use axum::{
    routing::{get, post},
    Router,
};
use exchange::Timestamp;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::Redoc;
use utoipa_redoc::Servable;
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;

mod book_event;
mod books;
mod markets;
mod order_request;
mod orders;
mod trades;

use utoipa::{Modify, OpenApi};

// #[derive(ToSchema)]
// pub type Timestamp = i64;

#[derive(OpenApi)]
#[openapi(
    paths(
        orders::post,
        markets::post,
    ),
    components(
        schemas(orders::OrderRequest, orders::TimeInForce, markets::Market),
    ),
    tags(
        (name = "exchange", description = "exchange thing")
    )
)]
pub struct ApiDoc;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .route("/orders", get(orders::get))
        .route("/markets/:slug", post(markets::post))
        .with_state(state)
}
