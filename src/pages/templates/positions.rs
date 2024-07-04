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
    market_title: String,
    event_title: String,
    side: String,
    position: String,
    last_price: String,
    market_value: String,
}

impl From<Position> for PositionAsHtml {
    fn from(position: Position) -> Self {
        let side = if position.position >= 0 { "Yes" } else { "No" }.to_string();
        Self {
            market_title: position.market_title,
            event_title: position.event_title,
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
                        SELECT market.title FROM market WHERE market.id = (
                            SELECT event.market_id FROM event WHERE event.id = position.event_id
                        )
                    ) as market_title,
                    (
                        SELECT event.title FROM event WHERE event.id = position.event_id
                    ) as event_title,
                    position.position,
                    (
                        SELECT trade.price FROM trade WHERE trade.event_id = position.event_id ORDER BY trade.id DESC LIMIT 1
                    ) AS last_price,
                    (
                        SELECT trade.price * ABS(position.position)
                        FROM trade WHERE trade.event_id = position.event_id ORDER BY trade.id DESC LIMIT 1
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
