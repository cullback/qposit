mod about;
mod orderbook;
mod home;
mod login;
mod markets;
mod orders;
mod profile;
mod signup;
mod templates;
pub use templates::orderbook::{OrderBook, PriceLevel};

use axum::{
    http::header,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};

use crate::app_state::AppState;

async fn get_pico_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../../static/pico.min.css"),
    )
}
async fn get_pico_colors() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../../static/pico.colors.css"),
    )
}
async fn get_htmx() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        include_str!("../../static/htmx.min.js"),
    )
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/pico.min.css", get(get_pico_css))
        .route("/pico.colors.min.css", get(get_pico_colors))
        .route("/htmx.min.js", get(get_htmx))
        .route("/", get(home::get))
        .route("/about", get(about::get))
        .route("/profile", get(profile::get))
        .route(
            "/login",
            get(login::get).post(login::post).delete(login::delete),
        )
        .route("/login/:session_id", delete(login::delete_by_id))
        .route("/signup", get(signup::get).post(signup::post))
        .route("/markets/:slug", get(markets::get))
        .route("/orders", post(orders::post))
        .route("/orders/:order_id", delete(orders::delete_by_id))
        .route("/orderbook", get(orderbook::sse_handler))
        .with_state(state)
}
