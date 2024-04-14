use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use exchange::{BookId, Position, UserId};
use orderbook::Price;
use sqlx::SqlitePool;
use tracing::warn;

use crate::auth::SessionExtractor;

#[derive(sqlx::FromRow)]
struct PositionThing {
    pub market_title: String,
    pub book_title: String,
    pub position: Position,
    pub last_price: Price,
    pub market_value: i32,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Component<'a> {
    username: &'a str,
    positions: Vec<PositionThing>,
}

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    Extension(db): Extension<SqlitePool>,
) -> impl IntoResponse {
    let Some(user) = user else {
        return Redirect::to("/").into_response();
    };

    let positions = match sqlx::query_as::<_, PositionThing>(
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
        .bind(user.id)
        .fetch_all(&db)
        .await {
            Ok(positions) => positions,
            Err(err) => {
            warn!(?err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let page = Component {
        username: &user.username,
        positions,
    }
    .render()
    .unwrap();
    Html(page).into_response()
}
