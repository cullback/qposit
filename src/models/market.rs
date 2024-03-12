use sqlx::SqlitePool;

use crate::app_state::Timestamp;

pub struct Market {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub status: String,
    pub create_time: Timestamp,
    pub expire_time: Timestamp,
}

pub async fn get_active_markets(db: &SqlitePool) -> Result<Vec<Market>, sqlx::Error> {
    sqlx::query_as!(
        Market,
        r#"
        SELECT
            id,
            title,
            description,
            status,
            create_time,
            expire_time
        FROM 'market'
        "#,
    )
    .fetch_all(db)
    .await
}
