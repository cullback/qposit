use exchange::BookId;
use sqlx::{prelude::FromRow, SqlitePool};

#[derive(Debug, FromRow)]
pub struct Book {
    pub id: BookId,
    pub market_id: i64,
    pub title: String,
    pub value: Option<u16>,
    pub last_trade_price: Option<u16>,
}

impl Book {
    pub async fn insert(&self, db: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO book (market_id, title, value)
            VALUES (?, ?, ?)",
            self.market_id,
            self.title,
            self.value,
        )
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid())
    }

    pub async fn get_all_for_market(
        db: &SqlitePool,
        market: i64,
    ) -> Result<Vec<Book>, sqlx::Error> {
        sqlx::query_as::<_, Book>(
            r#"
            SELECT
                book.id,
                book.market_id,
                book.title,
                book.value,
                (
                    SELECT trade.price
                    FROM trade
                    WHERE trade.book_id = book.id
                    ORDER BY trade.tick DESC
                    LIMIT 1
                ) AS last_trade_price
            FROM book
            WHERE book.market_id = ?;
            "#,
        )
        .bind(market)
        .fetch_all(db)
        .await
    }

    pub async fn get_active(db: &SqlitePool) -> Result<Vec<Book>, sqlx::Error> {
        sqlx::query_as::<_, Book>(
            r#"
            SELECT
                book.id,
                book.market_id,
                book.title,
                book.value,
                (
                    SELECT trade.price
                    FROM trade
                    WHERE trade.book_id = book.id
                    ORDER BY trade.tick DESC
                    LIMIT 1
                ) as last_trade_price
            FROM book
            "#,
        )
        .fetch_all(db)
        .await
    }
}
