use lobster::{BookId, Timestamp, UserId};
use lobster::{OrderId, Price, Quantity, Side};
use sqlx::sqlite::SqliteQueryResult;
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

impl From<&Order> for lobster::Order {
    fn from(order: &Order) -> Self {
        lobster::Order::new(
            order.id,
            order.remaining,
            order.price,
            Side::new(order.is_buy),
        )
    }
}

impl Order {
    /// Inserts a new order into the database.
    pub async fn new<E>(
        db: &mut E,
        created_at: Timestamp,
        book_id: BookId,
        user_id: UserId,
        order: lobster::Order,
    ) -> Result<i64, sqlx::Error>
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        let is_buy = order.side.is_buy();
        sqlx::query!(
            "INSERT INTO 'order' (id, created_at, book_id, user_id, quantity, remaining, price, is_buy, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'open')",
            order.id,
            created_at,
            book_id,
            user_id,
            order.quantity,
            order.quantity,
            order.price,
            is_buy,
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

    /// Sets the status of an order to cancelled.
    pub async fn cancel_by_id<E>(db: &mut E, id: OrderId) -> Result<SqliteQueryResult, sqlx::Error>
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        sqlx::query!("UPDATE 'order' SET status = 'cancelled' WHERE id = ?", id)
            .execute(db)
            .await
    }

    pub async fn cancel_for_book<E>(
        db: &mut E,
        book: BookId,
    ) -> Result<SqliteQueryResult, sqlx::Error>
    where
        for<'c> &'c mut E: Executor<'c, Database = Sqlite>,
    {
        sqlx::query!(
            "UPDATE 'order' SET status = 'cancelled' WHERE book_id = ? and status = 'open'",
            book
        )
        .execute(db)
        .await
    }
}
