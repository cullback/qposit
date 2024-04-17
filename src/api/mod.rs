use axum::{
    routing::{delete, get, post},
    Json, Router,
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

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        orders::post,
        orders::get,
        markets::post,
        feed::get,
        trades::get,
    ),
    components(
        schemas(orders::OrderRequest, orders::TimeInForce, orders::OrderResponse, markets::Market),
    ),
    tags(
        (name = "QPosit", description = "QPosit provides a number of Application Programming Interfaces (APIs) through HTTP and Websockets (WS).")
    ),
)]

pub struct ApiDoc;

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
