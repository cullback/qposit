use lobster::Balance;
use lobster::Timestamp;
use lobster::UserId;
use sqlx::Executor;
use sqlx::Sqlite;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password_hash: String,
    pub created_at: Timestamp,
    pub balance: Balance,
    pub available: Balance,
}

impl User {
    /// Inserts the user into the database and returns the new user's ID.
    pub async fn new<'c, E: Executor<'c, Database = Sqlite>>(
        db: E,
        username: &str,
        password_hash: &str,
        created_at: Timestamp,
    ) -> Result<UserId, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO user (username, password_hash, created_at) VALUES (?, ?, ?)",
            username,
            password_hash,
            created_at
        )
        .execute(db)
        .await
        .map(|row| UserId::try_from(row.last_insert_rowid()).unwrap())
    }

    pub async fn get_by_id(db: &SqlitePool, id: UserId) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM 'user' WHERE id = ?")
            .bind(id)
            .fetch_one(db)
            .await
    }

    pub async fn get_by_username(db: &SqlitePool, username: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM 'user' WHERE username = ?")
            .bind(username)
            .fetch_one(db)
            .await
    }

    pub async fn get_with_nonzero_balances(db: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM user WHERE balance != 0")
            .fetch_all(db)
            .await
    }

    /// Deposits the given amount into the user's account.
    pub async fn deposit<'c, E: Executor<'c, Database = Sqlite>>(
        db: E,
        user_id: UserId,
        amount: Balance,
    ) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            "
            UPDATE user SET balance = balance + ?, available = available + ? WHERE id = ?
            ",
            amount,
            amount,
            user_id
        )
        .execute(db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        };

        Ok(())
    }
}
