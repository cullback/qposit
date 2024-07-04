use lobster::EventId;
use lobster::UserId;
use serde::Deserialize;
use serde::Serialize;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Executor;
use sqlx::QueryBuilder;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PositionParams {
    pub event_id: Option<u32>,
    pub user_id: Option<u32>,
}

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

    pub async fn get(
        pool: &SqlitePool,
        params: PositionParams,
    ) -> Result<Vec<Position>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT * from position WHERE position != 0");

        if let Some(event_id) = params.event_id {
            query.push(" AND event_id = ");
            query.push_bind(event_id);
        }
        if let Some(user_id) = params.user_id {
            query.push(" AND user_id = ");
            query.push_bind(user_id);
        }

        query.build_query_as::<Position>().fetch_all(pool).await
    }
}
