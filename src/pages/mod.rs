mod home;
mod not_found;
mod templates;

use axum::{http::header, response::IntoResponse, routing::get, Router};

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

pub fn router() -> Router {
    Router::new()
        .route("/pico.min.css", get(get_pico_css))
        .route("/pico.colors.min.css", get(get_pico_colors))
        .route("/htmx.min.js", get(get_htmx))
        .route("/", get(home::get))
        .fallback(not_found::get)
}
