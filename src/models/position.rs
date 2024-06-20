use lobster::BookId;
use lobster::UserId;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Executor;
use sqlx::Sqlite;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow, Debug)]
pub struct Position {
    pub user_id: UserId,
    pub book_id: BookId,
    pub position: i32,
}

impl Position {
    pub async fn get_non_zero(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM position WHERE position != 0")
            .fetch_all(pool)
            .await
    }

    pub async fn delete_for_book<E>(
        pool: &mut E,
        book: BookId,
    ) -> Result<SqliteQueryResult, sqlx::Error>
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        sqlx::query!("DELETE FROM position WHERE book_id = ?", book)
            .execute(pool)
            .await
    }
}
