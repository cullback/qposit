use exchange::Timestamp;
use exchange::UserId;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password_hash: String,
    pub created_at: Timestamp,
    pub balance: i64,
}

impl User {
    pub async fn get_by_id(db: &SqlitePool, id: UserId) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM 'user' WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await
    }

    pub async fn get_by_username(db: &SqlitePool, username: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM 'user' WHERE username = ?")
            .bind(username)
            .fetch_one(db)
            .await
    }

    pub async fn insert(&self, db: &SqlitePool) -> Result<UserId, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO user (username, password_hash, created_at) VALUES (?, ?, ?)",
            self.username,
            self.password_hash,
            self.created_at
        )
        .execute(db)
        .await
        .map(|row| UserId::try_from(row.last_insert_rowid()).unwrap())
    }

    pub async fn get_with_nonzero_balances(db: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM user WHERE balance != 0")
            .fetch_all(db)
            .await
    }
}
