use exchange::{BookId, Timestamp, UserId};
use orderbook::{OrderId, Price, Quantity};
use sqlx::{prelude::FromRow, Executor, Sqlite, SqlitePool};

#[derive(Debug, FromRow)]
pub struct Order {
    pub id: OrderId,
    pub created_at: Timestamp,
    pub book_id: BookId,
    pub user_id: UserId,
    pub quantity: Quantity,
    pub remaining: Quantity,
    pub price: Price,
    pub is_buy: bool,
    pub status: String,
}

impl Order {
    pub async fn insert<'c, E>(&self, db: E) -> Result<i64, sqlx::Error>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        sqlx::query!(
            "INSERT INTO 'order' (id, created_at, book_id, user_id, quantity, remaining, price, is_buy, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            self.id,
            self.created_at,
            self.book_id,
            self.user_id,
            self.quantity,
            self.remaining,
            self.price,
            self.is_buy,
            self.status,
        )
        .execute(db)
        .await
        .map(|row| row.last_insert_rowid())
    }

    pub async fn get_next_order_id(db: &SqlitePool) -> OrderId {
        let (order_id,): (OrderId,) = sqlx::query_as("SELECT MAX(id) FROM 'order'")
            .fetch_one(db)
            .await
            .unwrap();

        order_id + 1
    }

    pub async fn get_open_orders(db: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM 'order' WHERE status = 'open' ORDER BY price ASC, created_at ASC",
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_for_user(db: &SqlitePool, user: UserId) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM 'order' WHERE status = 'open' and user_id = ? ORDER BY price ASC, created_at ASC",
        )
        .bind(user)
        .fetch_all(db)
        .await
    }

    /// Returns the open orders of a book from lowest price to highest price.
    pub async fn get_open_for_book(
        db: &SqlitePool,
        book: BookId,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM 'order' WHERE status = 'open' and book_id = ? ORDER BY price ASC, created_at ASC",
        )
        .bind(book)
        .fetch_all(db)
        .await
    }
}
