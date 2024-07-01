#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod actors;
mod api;
mod app_state;
mod models;
mod pages;

use crate::actors::book_service::EventData;
use app_state::AppState;
use lobster::BookUpdate;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::net::SocketAddr;
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};
use tracing::info;

async fn connect_to_database() -> SqlitePool {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL not set");
    SqlitePoolOptions::new()
        .connect(&url)
        .await
        .expect("Failed to connect to database")
}

fn exit_on_panic() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        std::process::exit(1);
    }));
}

#[tokio::main]
async fn main() {
    exit_on_panic();

    tracing_subscriber::fmt().init();

    let pool = connect_to_database().await;

    let (cmd_send, cmd_receive) = mpsc::channel(32);
    let (feed_send, feed_receive) = broadcast::channel::<BookUpdate>(32);
    let (book_send, book_receive) = broadcast::channel::<EventData>(32);

    actors::writer::start_writer_service(pool.clone(), feed_receive.resubscribe());
    actors::matcher::start_matcher_service(pool.clone(), cmd_receive, feed_send);
    actors::book_service::start_book_service(pool.clone(), feed_receive.resubscribe(), book_send);

    let state = AppState::new(pool, cmd_send, feed_receive, book_receive);

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
