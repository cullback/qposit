use exchange::BookId;
use exchange::UserId;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow, Debug)]
pub struct Position {
    pub user_id: UserId,
    pub book_id: BookId,
    pub position: i32,
}

impl Position {
    pub async fn get_non_zero(pool: &SqlitePool) -> Result<Vec<Position>, sqlx::Error> {
        sqlx::query_as::<_, Position>("SELECT * FROM position WHERE position != 0")
            .fetch_all(pool)
            .await
    }
}
