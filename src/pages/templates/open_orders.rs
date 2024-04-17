use askama::Template;
use exchange::{BookId, UserId};
use orderbook::{OrderId, Price, Quantity};
use sqlx::SqlitePool;

#[derive(sqlx::FromRow)]
struct Order {
    id: OrderId,
    book_id: BookId,
    quantity: Quantity,
    price: Price,
    is_buy: bool,
    status: String,
}

#[derive(Template)]
#[template(path = "open_orders.html")]
pub struct OpenOrders {
    orders: Vec<Order>,
}

impl OpenOrders {
    pub async fn build(db: &SqlitePool, user: UserId) -> Self {
        let orders = sqlx::query_as::<_, Order>(
            r#"
                SELECT
                    id,
                    book_id,
                    quantity,
                    price,
                    is_buy,
                    status
                FROM 'order'
                WHERE user_id = ? AND status = 'open'
            "#,
        )
        .bind(user)
        .fetch_all(db)
        .await
        .unwrap();

        OpenOrders { orders }
    }
}
