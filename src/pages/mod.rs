mod about;
mod auth;
mod events;
mod home;
mod login;
mod orderbook;
mod orders;
mod signup;
mod templates;
mod users;

pub use templates::orderbook::OrderBook;

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
async fn get_main_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../../static/main.css"),
    )
}
async fn get_htmx() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        include_str!("../../static/htmx.min.js"),
    )
}
async fn get_htmx_ws() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        include_str!("../../static/htmx.ws.js"),
    )
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/pico.min.css", get(get_pico_css))
        .route("/pico.colors.min.css", get(get_pico_colors))
        .route("/main.css", get(get_main_css))
        .route("/htmx.min.js", get(get_htmx))
        .route("/htmx.ws.js", get(get_htmx_ws))
        .route("/", get(home::get))
        .route("/about", get(about::get))
        // .route("/profile", get(profile::get))
        .route("/users/:username", get(users::get))
        .route(
            "/login",
            get(login::get).post(login::post).delete(login::delete),
        )
        .route("/login/:session_id", delete(login::delete_by_id))
        .route("/signup", get(signup::get).post(signup::post))
        .route("/events/:slug", get(events::get))
        .route("/orders", post(orders::post))
        .route("/orders/:order_id", delete(orders::delete_by_id))
        .route("/orderbook", get(orderbook::get))
        .with_state(state)
}
