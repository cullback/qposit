#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod actors;
mod api;
mod app_state;
mod auth;
mod bootstrap;
mod models;

use app_state::AppState;
use axum::{Extension, Router};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::net::SocketAddr;
use tokio::{net::TcpListener, sync::mpsc};
use tracing::info;

/// Connects to the database using the `DATABASE_URL` environment variable.
async fn connect_db() -> SqlitePool {
    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL not set");
    SqlitePoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to connect to database")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    // let (md_producer, feed) = tokio::sync::broadcast::channel::<BookEvent>(32);
    let (cmd_producer, cmd_receiver) = mpsc::channel(32);

    let state = AppState::build(cmd_producer).await;

    // let engine = bootstrap::bootstrap_exchange(&state.database).await;

    // tokio::spawn(async move {
    //     matcher::run_market(engine, cmd_receiver, md_producer).await;
    // });

    let db = connect_db().await;

    let app = Router::new()
        .nest("/", api::router(state))
        .layer(Extension(db));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    info!("Starting server on {addr}");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
