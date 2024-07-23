use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn connect_to_database() -> SqlitePool {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL not set");
    SqlitePoolOptions::new()
        .connect(&url)
        .await
        .expect("Failed to connect to database")
}


/// Crashes the whole application if any task panics.
pub fn register_panic_hook() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        std::process::exit(1);
    }));
}