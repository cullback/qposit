use lobster::EventId;
use lobster::UserId;
use serde::Serialize;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Executor;
use sqlx::Sqlite;
use sqlx::SqliteExecutor;
use sqlx::SqlitePool;
use utoipa::ToSchema;

#[derive(sqlx::FromRow, Debug, ToSchema, Serialize)]
pub struct Position {
    pub user_id: UserId,
    pub event_id: EventId,
    /// The position. Positive is long, negative is short.
    pub position: i32,
}

impl Position {
    pub async fn get_non_zero(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM position WHERE position != 0")
            .fetch_all(pool)
            .await
    }

    pub async fn delete_for_event<E>(
        pool: &mut E,
        event_id: EventId,
    ) -> Result<SqliteQueryResult, sqlx::Error>
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        sqlx::query!("DELETE FROM position WHERE event_id = ?", event_id)
            .execute(pool)
            .await
    }

    pub async fn get_for_user(
        pool: &SqlitePool,
        user_id: UserId,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM position WHERE user_id = ? AND position != 0")
            .bind(user_id)
            .fetch_all(pool)
            .await
    }
}
