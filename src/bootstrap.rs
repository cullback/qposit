use exchange::Exchange;
use sqlx::SqlitePool;


/// Initializes the in-memory exchange data from the database.
pub async fn bootstrap_exchange(db: &SqlitePool) -> Exchange {
    let mut engine = Exchange::default();

    engine
}