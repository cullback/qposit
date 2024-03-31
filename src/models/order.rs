use exchange::{BookId, Timestamp, UserId};
use orderbook::{OrderId, Price, Quantity};
use sqlx::{prelude::FromRow, SqlitePool};

#[derive(Debug, FromRow)]
pub struct Order {
    pub id: OrderId,
    pub created_at: Timestamp,
    pub book_id: BookId,
    pub user_id: UserId,
    pub quantity: Quantity,
    pub filled_qty: Quantity,
    pub price: Price,
    pub is_buy: bool,
    pub status: String,
}

impl Order {
    pub async fn get_next_order_id(db: &SqlitePool) -> OrderId {
        let (order_id,): (OrderId,) = sqlx::query_as("SELECT MAX(id) FROM 'order'")
            .fetch_one(db)
            .await
            .unwrap();

        order_id + 1
    }

    pub async fn get_open_orders(db: &SqlitePool) -> Result<Vec<Order>, sqlx::Error> {
        sqlx::query_as::<_, Order>(
            r#"
        SELECT * FROM 'order' WHERE status = 'open'
        ORDER BY price ASC, created_at ASC
        "#,
        )
        .fetch_all(db)
        .await
    }
}
