use sqlx::{Executor, Sqlite};

use lobster::{Timestamp, UserId};

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct Invite {
    pub id: i64,
    pub code: String,
    pub used_by: Option<UserId>,
    pub created_by: UserId,
    pub created_at: Timestamp,
}

impl Invite {
    pub async fn check_and_claim<'c, E: Executor<'c, Database = Sqlite>>(
        db: E,
        code: &str,
        user_id: UserId,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Invite>(
            "UPDATE invite 
             SET used_by = ? 
             WHERE code = ? AND used_by IS NULL
             RETURNING *",
        )
        .bind(user_id)
        .bind(code)
        .fetch_optional(db)
        .await
    }
}
