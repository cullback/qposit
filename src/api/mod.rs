use axum::{routing::get, Router};

use crate::app_state::AppState;

mod books;
mod markets;
mod orders;
mod trades;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/orders", get(orders::get))
        .with_state(state)
}
