use sqlx::SqlitePool;

use crate::app_state::Timestamp;

pub struct UserRecord {
    pub id: u32,
    pub username: String,
    pub password_hash: String,
    pub create_time: i64,
}

pub async fn get_user_by_id(db: &SqlitePool, id: u32) -> Result<UserRecord, sqlx::Error> {
    sqlx::query_as!(
        UserRecord,
        r#"
        SELECT
            id as "id: u32",
            username,
            password_hash,
            create_time
        FROM 'user'
        WHERE id = ?
        "#,
        id
    )
    .fetch_one(db)
    .await
}

pub async fn get_user_by_username(
    db: &SqlitePool,
    username: &str,
) -> Result<UserRecord, sqlx::Error> {
    sqlx::query_as!(
        UserRecord,
        r#"
        SELECT
            id as "id: u32",
            username,
            password_hash,
            create_time
        FROM 'user'
        WHERE username = ?
        "#,
        username
    )
    .fetch_one(db)
    .await
}

pub async fn insert(
    db: &SqlitePool,
    username: &str,
    password_hash: &str,
    create_time: Timestamp,
) -> Result<u32, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO user (username, password_hash, create_time)
        VALUES (?, ?, ?)
        "#,
        username,
        password_hash,
        create_time
    )
    .execute(db)
    .await
    .map(|row| u32::try_from(row.last_insert_rowid()).unwrap())
}
