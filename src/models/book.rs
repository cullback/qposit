use sqlx::SqlitePool;

pub struct Book {
    pub id: i64,
    pub market_id: i64,
    pub title: String,
    pub status: String,
    pub value: Option<u16>,
    pub last_trade_price: Option<u16>,
}

impl Book {
    pub async fn insert(&self, db: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO book (market_id, title, status, value)
            VALUES (?, ?, ?, ?)",
            self.market_id,
            self.title,
            self.status,
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
        sqlx::query_as!(
            Book,
            r#"
            SELECT
                book.id,
                book.market_id,
                book.title,
                book.status,
                book.value as "value: u16",
                (
                    SELECT trade.price
                    FROM trade
                    WHERE trade.book_id = book.id
                    ORDER BY trade.tick DESC
                    LIMIT 1
                ) AS "last_trade_price: u16"
            FROM book
            WHERE book.market_id = ?;
            "#,
            market,
        )
        .fetch_all(db)
        .await
    }
}

// pub async fn get_books(db: &SqlitePool) -> Result<Vec<Book>, sqlx::Error> {
//     sqlx::query_as!(
//         Book,
//         r#"
//         SELECT
//             book.id,
//             book.market_id,
//             book.title,
//             book.status,
//             book.value as "value: u16",
//             (
//                 SELECT trade.price
//                 FROM trade
//                 WHERE trade.book_id = book.id
//                 ORDER BY trade.tick DESC
//                 LIMIT 1
//             ) AS "last_trade_price: u16"
//         FROM book;
//         "#,
//     )
//     .fetch_all(db)
//     .await
// }
