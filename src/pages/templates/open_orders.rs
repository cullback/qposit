use askama::Template;
use lobster::{EventId, UserId};
use lobster::{OrderId, Price, Quantity};
use sqlx::SqlitePool;

use super::format_price_to_string;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct Order {
    market_title: String,
    book_title: String,
    id: OrderId,
    event_id: EventId,
    quantity: Quantity,
    remaining: Quantity,
    price: Price,
    is_buy: bool,
    status: String,
}

struct OrderAsHtml {
    market_title: String,
    book_title: String,
    id: OrderId,
    quantity: Quantity,
    remaining: Quantity,
    price: String,
    side: String,
    #[allow(dead_code)]
    status: String,
}

impl From<Order> for OrderAsHtml {
    fn from(order: Order) -> Self {
        Self {
            market_title: order.market_title,
            book_title: order.book_title,
            id: order.id,
            quantity: order.quantity,
            remaining: order.remaining,
            price: format_price_to_string(order.price),
            side: if order.is_buy { "Yes" } else { "No" }.to_string(),
            status: order.status,
        }
    }
}

#[derive(Template)]
#[template(path = "open_orders.html")]
pub struct OpenOrders {
    orders: Vec<OrderAsHtml>,
}

impl OpenOrders {
    pub async fn build(db: &SqlitePool, user: UserId) -> Self {
        let orders = sqlx::query_as::<_, Order>(
            "
                SELECT
                    (
                        SELECT market.title FROM market WHERE market.id = (
                            SELECT book.market_id FROM book WHERE book.id = 'order'.event_id
                        )
                    ) as market_title,
                    (
                        SELECT book.title FROM book WHERE book.id = 'order'.event_id
                    ) as book_title,
                    id,
                    event_id,
                    quantity,
                    remaining,
                    price,
                    is_buy,
                    status
                FROM 'order'
                WHERE user_id = ? AND status = 'open'
            ",
        )
        .bind(user)
        .fetch_all(db)
        .await
        .unwrap();

        Self {
            orders: orders.into_iter().map(OrderAsHtml::from).collect(),
        }
    }
}
