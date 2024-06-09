#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    // clippy::arithmetic_side_effects,
    // clippy::as_conversions,
    clippy::expect_used,
    clippy::float_arithmetic,
    clippy::integer_division,
    clippy::unwrap_used,
)]
mod book_event;
mod math;
mod order_request;
mod orderbook;
mod reject_reason;

use std::collections::{hash_map::Entry, HashMap};

pub use book_event::{Action, BookEvent};
pub use order_request::{OrderRequest, TimeInForce};
pub use reject_reason::RejectReason;

pub use orderbook::{Fill, Order, OrderBook, OrderId, Price, Quantity, Side};

pub use math::trade_cost;

/// Balance in basis points.
pub type Balance = i64;

pub type Position = i32;

pub const RESOLVE_PRICE: Price = 10000;

/// UTC Timestamp in microseconds.
pub type Timestamp = i64;

pub type Tick = u32;

pub type BookId = u32;

pub type UserId = u32;

#[derive(Debug)]
struct OrderOwner {
    user_id: UserId,
    book_id: BookId,
}

#[derive(Debug, Default)]
struct BookDetails {
    next_tick: Tick,
    inner: OrderBook,
}

#[derive(Debug, Default)]
pub struct Exchange {
    balances: HashMap<UserId, Balance>,
    available: HashMap<UserId, Balance>,
    positions: HashMap<(UserId, BookId), Position>,
    order_owner: HashMap<OrderId, OrderOwner>,
    books: HashMap<BookId, BookDetails>,
    /// The order id to assign to the next accepted `Order`.
    next_order_id: OrderId,
}

impl Exchange {
    #[must_use]
    pub fn new(next_order_id: OrderId) -> Self {
        Self {
            next_order_id,
            ..Default::default()
        }
    }

    /// Adds a book to the exchange.
    ///
    /// Does nothing if the book already existed.
    pub fn add_book(
        &mut self,
        timestamp: Timestamp,
        book_id: BookId,
    ) -> Result<BookEvent, RejectReason> {
        let book = BookDetails::default();
        self.books.insert(book_id, book);
        Ok(BookEvent::add_book(timestamp, book_id))
    }

    /// Sets the position for a user in a book.
    /// Used on startup for initialization.
    pub fn set_position(&mut self, user: UserId, book: BookId, position: Position) {
        *self.positions.entry((user, book)).or_default() = position;
    }

    /// Function for initializing the book. Should only be called on startup.
    /// Can't be done using `submit_order` because we need to specify order id.
    ///
    /// # Panics
    ///
    /// Panics if the book does not exist or if the order is marketable.
    pub fn init_order(&mut self, user: UserId, book_id: BookId, order: Order) {
        #[allow(clippy::expect_used)]
        let book = self
            .books
            .get_mut(&book_id)
            .map(|book| &mut book.inner)
            .expect("Book does not exist for order");

        let fills = book.add(order);
        assert!(fills.is_empty());

        self.order_owner.insert(
            order.id,
            OrderOwner {
                user_id: user,
                book_id,
            },
        );

        let position = self.positions.entry((user, book_id)).or_default();
        let cost = trade_cost(*position, order.quantity, order.price, order.side);
        let available = self.available.entry(user).or_default();
        *available -= cost;
    }

    /// Deposits an amount into a users account.
    /// If the user is not present, they are added.
    pub fn deposit(&mut self, user: UserId, amount: Balance) {
        *self.balances.entry(user).or_default() += amount;
        *self.available.entry(user).or_default() += amount;
    }

    /// Resolves the book to a specific price and clears positions.
    ///
    /// # Errors
    ///
    /// - Returns `Err(RejectReason::BookNotFound)` if the book does not exist.
    /// - Returns `Err(RejectReason::InvalidPrice)` if the price is greater than `RESOLVE_PRICE`.
    pub fn resolve(
        &mut self,
        timestamp: Timestamp,
        book_id: BookId,
        user_id: UserId,
        price: Price,
    ) -> Result<BookEvent, RejectReason> {
        if price > RESOLVE_PRICE {
            Err(RejectReason::InvalidPrice)?;
        }

        let Some(book) = self.books.remove(&book_id) else {
            return Err(RejectReason::BookNotFound);
        };
        // cancel all orders in the book
        self.order_owner.retain(|_, order| order.book_id != book_id);

        self.positions.retain(|&(user, book), position| {
            if book != book_id {
                return true;
            }
            let change = if *position >= 0 {
                Balance::from(price) * Balance::from(*position)
            } else {
                Balance::from(RESOLVE_PRICE - price) * -Balance::from(*position)
            };
            *self.balances.entry(user).or_default() += change;
            false
        });

        let event = BookEvent::resolve(timestamp, book.next_tick, book_id, user_id, price);
        Ok(event)
    }

    /// Submits a new order to the exchange.
    ///
    /// # Errors
    ///
    /// - Returns `Err(RejectReason::BookNotFound)` if the book does not exist.
    /// - Returns `Err(RejectReason::InvalidPrice)` if the price is 0 or greater than or equal to `RESOLVE_PRICE`.
    /// - Returns `Err(RejectReason::InvalidQuantity)` if the quantity is 0.
    ///
    /// # Panics
    ///
    /// Panics if the order causes a trade that overflows `Balance` or `Position`.
    pub fn submit_order(
        &mut self,
        timestamp: Timestamp,
        user: UserId,
        order: OrderRequest,
    ) -> Result<BookEvent, RejectReason> {
        self.check_order(user, order)?;

        let id = self.next_order_id;
        self.next_order_id += 1;

        let order2 = Order::new(id, order.quantity, order.price, order.side);
        let qty_filled = self.add_order(user, order.book, order2);

        let book = self
            .books
            .get_mut(&order.book)
            .ok_or(RejectReason::BookNotFound)?; // infallible

        let quantity; // the quantity to report for the event
        if order.tif == TimeInForce::IOC {
            book.inner.remove(id);
            quantity = qty_filled;
        } else {
            let remaining = order.quantity - qty_filled;
            let position = self.positions.entry((user, order.book)).or_default();
            let cost = trade_cost(*position, remaining, order.price, order.side);
            let available = self.available.entry(user).or_default();
            *available -= cost;
            quantity = order.quantity;
            self.order_owner.insert(
                id,
                OrderOwner {
                    user_id: user,
                    book_id: order.book,
                },
            );
        }

        let tick = book.next_tick;
        book.next_tick = book.next_tick.wrapping_add(1);
        let event = BookEvent {
            time: timestamp,
            tick,
            book: order.book,
            user,
            action: Action::Add(Order::new(id, quantity, order.price, order.side)),
        };
        Ok(event)
    }

    fn add_order(&mut self, taker: UserId, book_id: BookId, order: Order) -> Quantity {
        #[allow(clippy::unwrap_used)]
        let book = self.books.get_mut(&book_id).unwrap(); // infallible

        let mut qty_filled = 0;
        let fills = book.inner.add(order);
        for fill in fills {
            self.handle_trade(book_id, taker, fill, order.side);
            qty_filled += fill.quantity;
            if fill.done {
                self.order_owner.remove(&fill.id);
            }
        }
        qty_filled
    }

    /// Cancels an order.
    ///
    /// # Errors
    ///
    /// - Returns `Err(RejectReason::OrderNotFound)` if the order does not exist or
    /// does not belong to the user.
    pub fn cancel_order(
        &mut self,
        timestamp: Timestamp,
        user: UserId,
        id: OrderId,
    ) -> Result<BookEvent, RejectReason> {
        let book_id = match self.order_owner.entry(id) {
            Entry::Occupied(entry) if entry.get().user_id == user => entry.remove().book_id,
            _ => return Err(RejectReason::OrderNotFound),
        };

        let book = self
            .books
            .get_mut(&book_id)
            .ok_or(RejectReason::BookNotFound)?; // infallible

        // update available balance
        let order = book.inner.remove(id).ok_or(RejectReason::OrderNotFound)?; // infallible
        let available = self.available.entry(user).or_default();
        let position = self.positions.entry((user, book_id)).or_default();
        let cost = trade_cost(*position, order.quantity, order.price, order.side);
        *available += cost;

        let event = BookEvent::remove(timestamp, book.next_tick, book_id, user, id);
        book.next_tick = book.next_tick.wrapping_add(1);

        Ok(event)
    }

    /// Updates balances, available, and positions based on an execution.
    fn handle_trade(&mut self, book: BookId, taker: UserId, fill: Fill, side: Side) {
        #[allow(clippy::cast_possible_wrap)]
        let signed_quantity = match side {
            Side::Buy => fill.quantity as i32,
            Side::Sell => -(fill.quantity as i32),
        };

        let user_id = taker;
        let balance = self.balances.entry(user_id).or_default();
        let position = self.positions.entry((user_id, book)).or_default();
        let cost = trade_cost(*position, fill.quantity, fill.price, side);
        let available = self.available.entry(user_id).or_default();
        *available -= cost;
        *balance -= cost;
        *position += signed_quantity;

        let user_id = self.order_owner[&fill.id].user_id;
        let balance = self.balances.entry(user_id).or_default();
        let position = self.positions.entry((user_id, book)).or_default();
        let cost = trade_cost(*position, fill.quantity, fill.price, !side);
        let available = self.available.entry(user_id).or_default();
        *available -= cost;
        *balance -= cost;
        *position -= signed_quantity;

        *available += trade_cost(*position, fill.quantity, fill.price, !side);
        *available += i64::from(RESOLVE_PRICE) * i64::from(fill.quantity);
    }

    fn check_order(&self, user: UserId, order: OrderRequest) -> Result<(), RejectReason> {
        if order.price == 0 || order.price >= RESOLVE_PRICE {
            Err(RejectReason::InvalidPrice)?;
        }
        if order.quantity == 0 {
            Err(RejectReason::InvalidQuantity)?;
        }
        let Some(book) = self.books.get(&order.book).map(|book| &book.inner) else {
            return Err(RejectReason::BookNotFound);
        };

        // capital risk check
        let available = *self.available.get(&user).unwrap_or(&0);
        let position = *self.positions.get(&(user, order.book)).unwrap_or(&0);
        let cost = trade_cost(position, order.quantity, order.price, order.side);
        if cost > available {
            Err(RejectReason::InsufficientFunds)?;
        }

        let is_marketable = match order.side {
            Side::Buy => book
                .best_ask()
                .map(|ask| order.price >= ask.price)
                .unwrap_or_default(),
            Side::Sell => book
                .best_bid()
                .map(|bid| order.price <= bid.price)
                .unwrap_or_default(),
        };

        if order.tif == TimeInForce::IOC && !is_marketable
            || order.tif == TimeInForce::POST && is_marketable
        {
            Err(RejectReason::IOCNotMarketable)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BookEvent, Exchange, OrderBook, OrderRequest, RejectReason, TimeInForce, RESOLVE_PRICE,
    };

    /// Set up exchange with two users with balances and one book.
    fn setup_default_scenario() -> Exchange {
        let user1 = 1;
        let user2 = 2;
        let book = 1;
        let mut exch = Exchange::default();
        exch.add_book(0, book).unwrap();
        exch.deposit(user1, 10 * i64::from(RESOLVE_PRICE));
        exch.deposit(user2, 10 * i64::from(RESOLVE_PRICE));
        exch
    }

    #[test]
    fn test_submit_ioc_then_cancel() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let order = OrderRequest::buy(1, 10, 200, TimeInForce::IOC);
        let event = exch.submit_order(time, 1, order);
        assert_eq!(event, Err(RejectReason::IOCNotMarketable));

        let event = exch.cancel_order(time, 1, 0);
        assert_eq!(event, Err(RejectReason::OrderNotFound));
    }

    #[test]
    fn test_submit_gtc_then_cancel_twice() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let user = 1;
        let order = OrderRequest::buy(book, 10, 200, TimeInForce::GTC);
        let event = exch.submit_order(time, user, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 0, book, user, 0, 10, 200)));

        let event = exch.cancel_order(time, user, 0);
        assert_eq!(event, Ok(BookEvent::remove(time, 1, book, user, 0)));

        let event = exch.cancel_order(time, user, 0);
        assert_eq!(event, Err(RejectReason::OrderNotFound));
    }

    #[test]
    fn test_cancel_traded_order() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let bob = 1;
        let cat = 2;

        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 3, 4000)));

        let order = OrderRequest::buy(book, 5, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 1, book, cat, 1, 5, 4000)));

        let event = exch.cancel_order(time, bob, 0);
        assert_eq!(event, Err(RejectReason::OrderNotFound));
    }

    #[test]
    fn test_queue_priority() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let bob = 1;
        let cat = 2;

        // this order should trade first
        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 3, 4000)));

        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 1, book, bob, 1, 3, 4000)));

        let order = OrderRequest::buy(book, 5, 4000, TimeInForce::GTC);
        assert!(exch.submit_order(time, cat, order).is_ok());

        let event = exch.cancel_order(time, bob, 0);
        assert_eq!(event, Err(RejectReason::OrderNotFound));
        // assert_eq!(event, Ok(BookEvent::remove(time, 2, book, bob, 0)));
    }

    #[test]
    fn test_unmarketable_ioc() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let user = 1;
        let order = OrderRequest::sell(book, 1, 5500, TimeInForce::GTC);
        let event = exch.submit_order(time, user, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, user, 0, 1, 5500)));

        let order = OrderRequest::buy(book, 1, 5400, TimeInForce::IOC);
        let event = exch.submit_order(time, user, order);
        assert_eq!(event, Err(RejectReason::IOCNotMarketable));
    }

    #[test]
    fn test_self_trade() {
        let mut exch = setup_default_scenario();
        let bob = 1;
        let time = 0;
        let book = 1;

        let order = OrderRequest::sell(book, 5, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 5, 4000)));

        assert_eq!(exch.available[&bob], 70000);

        let order = OrderRequest::buy(book, 2, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 1, book, bob, 1, 2, 4000)));

        assert_eq!(exch.balances[&bob], 100000);
        assert_eq!(exch.available[&bob], 82000);
        assert_eq!(exch.positions[&(bob, book)], 0);

        exch.cancel_order(time, bob, 0);
        assert_eq!(exch.available[&bob], 100000);
    }

    #[test]
    fn trade_multiple_levels() {
        let mut exch = setup_default_scenario();
        let time = 0;
        let book = 1;
        let bob = 1;
        let cat = 2;

        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 3, 4000)));
        let order = OrderRequest::sell(book, 3, 4200, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 1, book, bob, 1, 3, 4200)));
        let order = OrderRequest::sell(book, 3, 4100, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 2, book, bob, 2, 3, 4100)));
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 3, book, bob, 3, 3, 4100)));

        assert_eq!(exch.available[&bob], 100000 - 70800);
        assert_eq!(exch.balances[&bob], 100000);

        let order = OrderRequest::buy(book, 7, 4300, TimeInForce::GTC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 4, book, cat, 4, 7, 4300)));

        assert_eq!(exch.balances[&bob], 58400);
        assert_eq!(exch.balances[&cat], 71600);
        // assert_eq!(exch.available[&bob], 29200/*100000 - 70800 + 41600*/);
        // assert_eq!(exch.available[&bob], 29200/*100000 - 17400 - 11800*/);
        assert_eq!(exch.positions[&(bob, book)], -7);
        assert_eq!(exch.positions[&(cat, book)], 7);
    }

    #[test]
    fn trade_multiple_one_cancelled_buy() {
        let mut exch = setup_default_scenario();
        let time = 0;
        let book = 1;
        let bob = 1;
        let cat = 2;

        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        assert!(exch.submit_order(time, bob, order).is_ok());
        assert!(exch.submit_order(time, bob, order).is_ok());
        assert!(exch.submit_order(time, bob, order).is_ok());
        let order = OrderRequest::sell(book, 3, 4100, TimeInForce::GTC);
        assert!(exch.submit_order(time, bob, order).is_ok());
        assert!(exch.cancel_order(time, bob, 1).is_ok());

        let order = OrderRequest::buy(book, 7, 4100, TimeInForce::GTC);
        assert!(exch.submit_order(time, cat, order).is_ok());

        assert_eq!(exch.positions[&(bob, book)], -7);
        assert_eq!(exch.positions[&(cat, book)], 7);
        assert_eq!(exch.balances[&bob], 58100);
        assert_eq!(exch.balances[&cat], 71900);
    }

    #[test]
    fn trade_multiple_one_cancelled_sell() {
        let mut exch = setup_default_scenario();
        let time = 0;
        let book = 1;
        let bob = 1;
        let cat = 2;

        let order = OrderRequest::buy(book, 3, 40, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 0, book, bob, 0, 3, 40)));
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 1, book, bob, 1, 3, 40)));
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 2, book, bob, 2, 3, 40)));

        let order = OrderRequest::buy(book, 3, 39, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 3, book, bob, 3, 3, 39)));

        let event = exch.cancel_order(time, bob, 1);
        assert_eq!(event, Ok(BookEvent::remove(time, 4, book, bob, 1)));

        let order = OrderRequest::sell(book, 7, 39, TimeInForce::GTC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 5, book, cat, 4, 7, 39)));

        assert_eq!(exch.positions[&(bob, book)], 7);
        assert_eq!(exch.positions[&(cat, book)], -7);
        assert_eq!(exch.balances[&bob], 99721);
        assert_eq!(exch.balances[&cat], 30279);
    }

    #[test]
    fn test_resolve() {
        let mut exch = setup_default_scenario();
        let time = 0;
        let book = 1;
        let bob = 1;
        let cat = 2;

        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 3, 4000)));

        let order = OrderRequest::buy(book, 5, 4000, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 1, book, cat, 1, 3, 4000)));

        assert_eq!(exch.positions[&(bob, book)], -3);
        assert_eq!(exch.positions[&(cat, book)], 3);

        let event = exch.resolve(time, book, bob, 7000);
        assert_eq!(event, Ok(BookEvent::resolve(time, 2, book, bob, 7000)));
        assert!(!exch.positions.contains_key(&(bob, book)));
        assert!(!exch.positions.contains_key(&(cat, book)));
        assert_eq!(exch.balances[&bob], 91000);
        assert_eq!(exch.balances[&cat], 109000);

        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Err(RejectReason::BookNotFound));
    }

    #[test]
    fn trade_back_and_forth() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let bob = 1;
        let cat = 2;

        // bob places resting order
        let order = OrderRequest::sell(book, 10, 5250, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 10, 5250)));

        // cat submits order
        let order = OrderRequest::buy(book, 1, 5250, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 1, book, cat, 1, 1, 5250)));

        // bob places resting order
        let order = OrderRequest::buy(book, 10, 4750, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 2, book, bob, 2, 10, 4750)));

        // cat submits order
        let order = OrderRequest::sell(book, 1, 4750, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 3, book, cat, 3, 1, 4750)));

        assert_eq!(exch.balances[&bob], 100500);
        assert_eq!(exch.balances[&cat], 99500);
    }

    #[test]
    fn trade_with_top_of_book() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let bob = 1;
        let cat = 2;

        // bob places resting order
        let order = OrderRequest::sell(book, 5, 7000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 5, 7000)));

        let order = OrderRequest::buy(book, 5, 6000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 1, book, bob, 1, 5, 6000)));

        assert_eq!(exch.available[&bob], 100000 - 45000);

        let order = OrderRequest::buy(book, 1, 9999, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 2, book, cat, 2, 1, 9999)));
        let order = OrderRequest::sell(book, 1, 1, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 3, book, cat, 3, 1, 1)));

        assert_eq!(exch.balances[&bob], 101000);
        assert_eq!(exch.balances[&cat], 99000);
        assert_eq!(exch.available[&bob], 65000);
        assert_eq!(exch.available[&cat], 99000);

        assert!(exch.cancel_order(time, bob, 0).is_ok());
        assert!(exch.cancel_order(time, bob, 1).is_ok());

        assert_eq!(exch.available[&bob], 101000);
    }

    #[test]
    fn test_available() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let bob = 1;
        let cat = 2;

        // bob places resting order
        let order = OrderRequest::sell(book, 5, 7000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 5, 7000)));

        assert_eq!(exch.available[&bob], 85000); // 100000 - 15000

        let order = OrderRequest::buy(book, 1, 9999, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 1, book, cat, 1, 1, 9999)));

        assert_eq!(exch.available[&bob], 85000); // 85000 - 3000 + 3000
        assert_eq!(exch.balances[&bob], 97000);
        assert_eq!(exch.balances[&cat], 93000);
        assert_eq!(exch.available[&cat], 93000);

        assert!(exch.cancel_order(time, bob, 0).is_ok());
        assert_eq!(exch.available[&bob], 97000);
        assert_eq!(exch.available[&cat], 93000);
    }

    #[test]
    fn test_available2() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let bob = 1;
        let cat = 2;

        // bob places resting order
        let order = OrderRequest::buy(book, 5, 7000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 0, book, bob, 0, 5, 7000)));

        assert_eq!(exch.available[&bob], 65000); // 100000 - 35000

        let order = OrderRequest::sell(book, 1, 1, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 1, book, cat, 1, 1, 1)));

        assert_eq!(exch.balances[&bob], 93000);
        assert_eq!(exch.available[&bob], 65000);

        assert_eq!(exch.balances[&cat], 97000);
        assert_eq!(exch.available[&cat], 97000);

        assert!(exch.cancel_order(time, bob, 0).is_ok());
        assert_eq!(exch.available[&bob], 93000);
        assert_eq!(exch.available[&cat], 97000);
    }

    #[test]
    fn test_available3() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let bob = 1;
        let cat = 2;

        // bob places resting order
        let order = OrderRequest::sell(book, 5, 6000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 0, book, bob, 0, 5, 6000)));

        let order = OrderRequest::buy(book, 5, 5000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 1, book, bob, 1, 5, 5000)));

        assert_eq!(exch.available[&bob], 55000); // 100000 - 45000

        let order = OrderRequest::buy(book, 1, 9999, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::buy(time, 2, book, cat, 2, 1, 9999)));

        assert_eq!(exch.available[&cat], 94000);
        assert_eq!(exch.balances[&cat], 94000);

        // assert_eq!(exch.available[&bob], 94000);
        assert_eq!(exch.available[&bob], 54000);
        assert_eq!(exch.balances[&bob], 96000);

        let order = OrderRequest::sell(book, 3, 1, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(BookEvent::sell(time, 3, book, cat, 3, 3, 1)));

        assert_eq!(exch.balances[&cat], 89000);
        assert_eq!(exch.available[&cat], 89000);

        // assert_eq!(exch.balances[&bob], 101000);
        // assert_eq!(exch.available[&bob], 65000);

        // assert!(exch.cancel_order(time, bob, 0).is_ok());
        // assert!(exch.cancel_order(time, bob, 1).is_ok());

        // assert_eq!(exch.available[&bob], 101000);
    }
}
