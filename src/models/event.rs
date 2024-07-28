use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use utoipa::ToSchema;

/// An event is a collection of markets.
#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, ToSchema)]
pub struct Event {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub created_at: i64,
    pub event_time: i64,
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
            "INSERT INTO event (slug, title, description, created_at, event_time)
            VALUES (?, ?, ?, ?, ?)",
            self.slug,
            self.title,
            self.description,
            self.created_at,
            self.event_time,
        )
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid())
    }
}
