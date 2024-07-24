use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordVerifier;
use lobster::Balance;
use lobster::Timestamp;
use lobster::UserId;
use serde::Serialize;
use sqlx::Executor;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use tracing::error;

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    #[serde(skip)]
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

    /// Checks a usernames+password combination using the database and returns the user if it is valid.
    /// Returns `None` if the user does not exist or the password is incorrect.
    pub async fn check_login(pool: &SqlitePool, username: &str, password: &str) -> Option<User> {
        match User::get_by_username(pool, username).await {
            Ok(user) => {
                let parsed_hash =
                    PasswordHash::new(&user.password_hash).expect("Failed to parsh hash");
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed_hash)
                    .ok()
                    .map(|()| user)
            }
            Err(sqlx::Error::RowNotFound) => return None,
            Err(err) => {
                error!(err = ?err, "Failed to get user");
                return None;
            }
        }
    }
}
