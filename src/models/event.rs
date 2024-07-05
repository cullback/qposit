use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use utoipa::ToSchema;

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, ToSchema)]
pub struct Event {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub created_at: i64,
    pub expires_at: i64,
}

impl Event {
    pub async fn get_by_slug(db: &SqlitePool, slug: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM 'event' WHERE slug = ?")
            .bind(slug)
            .fetch_one(db)
            .await
    }
    pub async fn get_active_events(db: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM 'event'")
            .fetch_all(db)
            .await
    }

    pub async fn insert(&self, db: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO event (slug, title, description, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?)",
            self.slug,
            self.title,
            self.description,
            self.created_at,
            self.expires_at,
        )
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid())
    }
}
