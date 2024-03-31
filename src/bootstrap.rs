use exchange::{Exchange, OrderRequest, TimeInForce};
use sqlx::SqlitePool;

use crate::models::{book::Book, order::Order, position::Position, user::User};

/// Initializes the in-memory exchange data from the database.
pub async fn bootstrap_exchange(db: &SqlitePool) -> Exchange {
    let next_order_id = Order::get_next_order_id(db).await;

    let mut engine = Exchange::new(next_order_id);

    for user in User::get_with_nonzero_balances(db).await.unwrap() {
        engine.deposit(user.id, user.balance);
    }

    for book in Book::get_active(db).await.unwrap() {
        engine.add_book(book.id, 100);
    }

    for position in Position::get_non_zero(db).await.unwrap() {
        engine.set_position(position.user_id, position.book_id, position.position);
    }

    for order in Order::get_open_orders(db).await.unwrap() {
        let req = OrderRequest::new(
            order.book_id,
            order.quantity,
            order.price,
            order.is_buy,
            TimeInForce::GTC,
        );
        engine.init_order(order.id, order.user_id, req);
    }

    engine
}
