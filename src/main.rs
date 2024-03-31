#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod actors;
mod api;
mod app_state;
mod auth;
mod bootstrap;
mod models;
mod pages;

use crate::actors::matcher;
use app_state::AppState;
use axum::{Extension, Router};
use exchange::BookEvent;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::net::SocketAddr;
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};
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

    let db = connect_db().await;

    // let (md_producer, feed) = tokio::sync::broadcast::channel::<BookEvent>(32);
    let (cmd_send, cmd_receive) = mpsc::channel(32);
    let (feed_send, feed_receive) = broadcast::channel::<BookEvent>(32);
    let state = AppState::build(cmd_send, feed_receive).await;
    let engine = bootstrap::bootstrap_exchange(&db).await;

    tokio::spawn(async move {
        matcher::run_matcher(engine, cmd_receive, feed_send).await;
    });

    let app = pages::router()
        .merge(api::router(state))
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
