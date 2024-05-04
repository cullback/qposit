use askama::Template;
use exchange::UserId;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow, Debug)]
struct Position {
    market_title: String,
    book_title: String,
    position: i32,
    last_price: f32,
    market_value: f32,
}

#[derive(Template)]
#[template(path = "open_positions.html")]
pub struct Positions {
    positions: Vec<Position>,
}

impl Positions {
    pub async fn build(db: &SqlitePool, user: UserId) -> Self {
        let positions = sqlx::query_as::<_, Position>(
           "
                SELECT
                    (
                        SELECT market.title FROM market WHERE market.id = (
                            SELECT book.market_id FROM book WHERE book.id = position.book_id
                        )
                    ) as market_title,
                    (
                        SELECT book.title FROM book WHERE book.id = position.book_id
                    ) as book_title,
                    position.position,
                    (
                        SELECT CAST(trade.price AS REAL) FROM trade WHERE trade.book_id = position.book_id ORDER BY trade.id DESC LIMIT 1
                    ) AS last_price,
                    (
                        SELECT cast(trade.price * ABS(position.position) AS REAL)
                        FROM trade WHERE trade.book_id = position.book_id ORDER BY trade.id DESC LIMIT 1
                    ) AS market_value
                FROM position WHERE user_id = ? AND position.position != 0
                ",
            )
            .bind(user)
            .fetch_all(db)
            .await.unwrap();
        Self { positions }
    }
}
