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

#[derive(OpenApi)]
#[openapi(
    paths(
        orders::post,
        markets::post,
        book_event::ws_handler,
    ),
    components(
        schemas(orders::OrderRequest, orders::TimeInForce, markets::Market),
    ),
    tags(
        (name = "QPosit", description = "Prediction market.")
    ),
)]

pub struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let apiv1 = Router::new()
        .route("/orders", get(orders::get).post(orders::post))
        .route("/markets/:slug", post(markets::post))
        .route("/ws", get(book_event::ws_handler));

    Router::new()
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .nest("/api/v1", apiv1)
        .with_state(state)
}
