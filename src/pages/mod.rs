mod about;
mod home;
mod login;
mod market;
mod not_found;
mod profile;
mod signup;
mod templates;

use axum::{
    http::header,
    response::IntoResponse,
    routing::{delete, get},
    Router,
};

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
        .route("/about", get(about::get))
        .route("/profile", get(profile::get))
        .route(
            "/login",
            get(login::get).post(login::post).delete(login::delete),
        )
        .route("/login/:session_id", delete(login::delete_by_id))
        .route("/signup", get(signup::get).post(signup::post))
        .route("/market/:slug", get(market::get))
        .fallback(not_found::get)
}
