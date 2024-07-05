use askama::Template;
use lobster::Price;
use lobster::UserId;
use sqlx::SqlitePool;

use super::format_price_to_string;

#[derive(sqlx::FromRow, Debug)]
struct Position {
    market_title: String,
    event_title: String,
    position: i32,
    last_price: Price,
    market_value: u32,
}

struct PositionAsHtml {
    event_title: String,
    market_title: String,
    side: String,
    position: String,
    last_price: String,
    market_value: String,
}

impl From<Position> for PositionAsHtml {
    fn from(position: Position) -> Self {
        let side = if position.position >= 0 { "Yes" } else { "No" }.to_string();
        Self {
            event_title: position.event_title,
            market_title: position.market_title,
            side,
            position: format!("{}", position.position.abs()),
            last_price: format_price_to_string(position.last_price),
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
                        SELECT event.title FROM event WHERE event.id = (
                            SELECT market.event_id FROM market WHERE market.id = position.market_id
                        )
                    ) as event_title,
                    (
                        SELECT market.title FROM market WHERE market.id = position.market_id
                    ) as market_title,
                    position.position,
                    (
                        SELECT trade.price FROM trade WHERE trade.market_id = position.market_id ORDER BY trade.id DESC LIMIT 1
                    ) AS last_price,
                    (
                        SELECT trade.price * ABS(position.position)
                        FROM trade WHERE trade.market_id = position.market_id ORDER BY trade.id DESC LIMIT 1
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
