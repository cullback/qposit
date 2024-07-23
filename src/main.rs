#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod api;
mod app_state;
mod models;
mod services;
mod util;
mod web;

use crate::services::book_service::MarketData;
use app_state::AppState;
use lobster::BookUpdate;
use std::net::SocketAddr;
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};
use tracing::info;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use util::{connect_to_database, register_panic_hook};

fn configure_logging() {
    let file_appender = RollingFileAppender::new(Rotation::HOURLY, "logs", "qposit.log");

    let subscriber = tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[tokio::main]
async fn main() {
    register_panic_hook();

    configure_logging();

    let pool = connect_to_database().await;

    let (cmd_send, cmd_receive) = mpsc::channel(32);
    let (feed_send, feed_receive) = broadcast::channel::<BookUpdate>(32);
    let (book_send, book_receive) = broadcast::channel::<MarketData>(32);

    services::writer::start_writer_service(pool.clone(), feed_receive.resubscribe());
    services::matcher::start_matcher_service(pool.clone(), cmd_receive, feed_send);
    services::book_service::start_book_service(pool.clone(), feed_receive.resubscribe(), book_send);

    let state = AppState::new(pool, cmd_send, feed_receive, book_receive);

    let app = web::router(state.clone()).merge(api::router(state));

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
