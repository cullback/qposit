use sqlx::SqlitePool;

use exchange::Timestamp;

#[derive(sqlx::FromRow)]
pub struct Market {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}

impl Market {
    pub async fn get_active_markets(db: &SqlitePool) -> Result<Vec<Market>, sqlx::Error> {
        sqlx::query_as::<_, Market>(r#"SELECT * FROM 'market'"#)
            .fetch_all(db)
            .await
    }

    pub async fn insert(&self, db: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO market (slug, title, description, status, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?)",
            self.slug,
            self.title,
            self.description,
            self.status,
            self.created_at,
            self.expires_at,
        )
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid())
    }
}
