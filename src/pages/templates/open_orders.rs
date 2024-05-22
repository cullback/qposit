use askama::Template;
use lobster::{BookId, UserId};
use lobster::{OrderId, Price, Quantity};
use sqlx::SqlitePool;

#[derive(sqlx::FromRow)]
struct Order {
    market_title: String,
    book_title: String,
    id: OrderId,
    book_id: BookId,
    quantity: Quantity,
    price: Price,
    is_buy: bool,
    status: String,
}

struct OrderAsHtml {
    market_title: String,
    book_title: String,
    id: OrderId,
    quantity: Quantity,
    price: String,
    side: String,
    status: String,
}

impl From<Order> for OrderAsHtml {
    fn from(order: Order) -> Self {
        Self {
            market_title: order.market_title,
            book_title: order.book_title,
            id: order.id,
            quantity: order.quantity,
            price: format!("{:.2}", order.price as f32 / 100.0),
            side: if order.is_buy { "Buy" } else { "Sell" }.to_string(),
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
                            SELECT book.market_id FROM book WHERE book.id = 'order'.book_id
                        )
                    ) as market_title,
                    (
                        SELECT book.title FROM book WHERE book.id = 'order'.book_id
                    ) as book_title,
                    id,
                    book_id,
                    quantity,
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
