use askama::Template;
use lobster::UserId;
use lobster::Price;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow, Debug)]
struct Position {
    market_title: String,
    book_title: String,
    position: i32,
    last_price: Price,
    market_value: u32,
}

struct PositionAsHtml {
    market_title: String,
    book_title: String,
    position: String,
    last_price: String,
    market_value: String,
}

impl From<Position> for PositionAsHtml {
    fn from(position: Position) -> Self {
        Self {
            market_title: position.market_title,
            book_title: position.book_title,
            position: format!("{}", position.position),
            last_price: format!("{:.2}", position.last_price as f32 / 100.0),
            market_value: format!("{:.2}", position.market_value as f32 / 10000.0),
        }
    }
}

#[derive(Template)]
#[template(path = "open_positions.html")]
pub struct Positions {
    positions: Vec<PositionAsHtml>,
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
                        SELECT trade.price FROM trade WHERE trade.book_id = position.book_id ORDER BY trade.id DESC LIMIT 1
                    ) AS last_price,
                    (
                        SELECT trade.price * ABS(position.position)
                        FROM trade WHERE trade.book_id = position.book_id ORDER BY trade.id DESC LIMIT 1
                    ) AS market_value
                FROM position WHERE user_id = ? AND position.position != 0
                ",
            )
            .bind(user)
            .fetch_all(db)
            .await.unwrap();
        Self {
            positions: positions.into_iter().map(|p| p.into()).collect(),
        }
    }
}
