#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod actors;
mod api;
mod app_state;
mod auth;
mod models;
mod pages;

use crate::actors::matcher;
use app_state::AppState;
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

    let (cmd_send, cmd_receive) = mpsc::channel(32);
    let (feed_send, feed_receive) = broadcast::channel::<BookEvent>(32);

    tokio::spawn({
        let db = db.clone();
        let feed_receive = feed_receive.resubscribe();
        async move { actors::writer::run_persistor(db, feed_receive).await }
    });
    tokio::spawn({
        let db = db.clone();
        async move {
            matcher::run_matcher(db, cmd_receive, feed_send).await;
        }
    });

    let state = AppState::new(db, cmd_send, feed_receive);

    let app = pages::router(state.clone()).merge(api::router(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    info!("Starting server on {addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start server");
}
