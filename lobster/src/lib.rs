//! Prediction market matching engine.
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    // clippy::expect_used,
    clippy::float_arithmetic,
    clippy::integer_division,
    clippy::unwrap_used,
)]
mod accounting;
mod book_details;
mod market_update;
mod order_request;
mod orderbook;
mod reject_reason;

use std::collections::{hash_map::Entry, HashMap};

use book_details::BookDetails;
pub use market_update::{Action, MarketUpdate};

pub use order_request::{OrderRequest, TimeInForce};
pub use reject_reason::RejectReason;

pub use orderbook::{Fill, Order, OrderBook, OrderId, Price, Quantity, Side};

pub use accounting::{PortfolioManager, RESOLVE_PRICE};

/// Balance in basis points.
pub type Balance = i64;

/// Signed position, tracked per book.
pub type Position = i32;

/// UTC Timestamp in microseconds.
pub type Timestamp = i64;

/// Monotonically increasing sequence number, tracked per book.
pub type Tick = u32;

/// Unique book identifier.
pub type MarketId = u32;

/// Unique user identifier.
pub type UserId = u32;

pub type MatcherResult = Result<MarketUpdate, RejectReason>;

#[derive(Debug)]
struct OrderOwner {
    user_id: UserId,
    event_id: MarketId,
}

#[derive(Debug, Default)]
pub struct Exchange {
    /// Tracks user balances and positions.
    manager: PortfolioManager,
    orderbooks: HashMap<MarketId, BookDetails>,
    order_owner: HashMap<OrderId, OrderOwner>,
    /// The order id to assign to the next accepted `Order`.
    next_order_id: OrderId,
}

impl Exchange {
    fn build_new_order(&mut self, quantity: Quantity, price: Price, side: Side) -> Order {
        let id = self.next_order_id;
        self.next_order_id = self.next_order_id.wrapping_add(1);
        Order::new(id, quantity, price, side)
    }

    /// Deposits an amount into a users account.
    /// If the user is not present, they are added.
    pub fn deposit(&mut self, user: UserId, amount: Balance) {
        self.manager.deposit(user, amount);
    }

    /// Constructs an exchange from an initial state.
    /// Orders must be sorted by order id.
    ///
    /// # Panics
    ///
    /// - Panics if there's a marketable order, or balance exceeds exposure
    #[must_use]
    pub fn from_state(
        next_order_id: OrderId,
        balances: &HashMap<UserId, Balance>,
        positions: &HashMap<(UserId, MarketId), Position>,
        orders: &[(UserId, MarketId, Order)],
        events: &[MarketId],
    ) -> Self {
        let mut tracker = PortfolioManager::new(balances, positions);

        let mut orderbooks: HashMap<MarketId, BookDetails> = events
            .iter()
            .map(|&event_id| (event_id, BookDetails::default()))
            .collect();

        let mut order_owner = HashMap::new();
        for &(user_id, event_id, order) in orders {
            tracker.add_resting_order(user_id, event_id, order);
            order_owner.insert(order.id, OrderOwner { user_id, event_id });
            assert!(orderbooks
                .get_mut(&event_id)
                .expect("Expected book to exist")
                .add(order)
                .is_empty());
        }

        Self {
            manager: tracker,
            orderbooks,
            order_owner,
            next_order_id,
        }
    }

    /// Adds a book to the exchange.
    ///
    /// # Errors
    ///
    /// - Returns `Err(RejectReason::BookAlreadyExists)` if the book already exists.
    pub fn add_event(
        &mut self,
        timestamp: Timestamp,
        event_id: MarketId,
    ) -> MatcherResult {
        let book = BookDetails::default();
        match self.orderbooks.entry(event_id) {
            Entry::Occupied(_) => Err(RejectReason::MarketAlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(book);
                Ok(MarketUpdate::add_book(timestamp, event_id))
            }
        }
    }

    /// Resolves a book to the specified price. Cancels open orders and zeroes positions.
    ///
    /// TODO: this function is inefficient
    ///
    /// # Errors
    ///
    /// - Returns `Err(RejectReason::BookNotFound)` if the book does not exist.
    /// - Returns `Err(RejectReason::InvalidPrice)` if the price is greater than `RESOLVE_PRICE`.
    pub fn resolve(
        &mut self,
        timestamp: Timestamp,
        event_id: MarketId,
        price: Price,
    ) -> MatcherResult {
        if price > RESOLVE_PRICE {
            return Err(RejectReason::InvalidPrice);
        }
        let Some(mut book) = self.orderbooks.remove(&event_id) else {
            return Err(RejectReason::MarketNotFound);
        };

        self.order_owner.retain(|_, order| order.event_id != event_id);
        self.manager.resolve(event_id, price);
        let event = book.resolve_event(timestamp, event_id, price);
        Ok(event)
    }

    /// Submits a new order to the exchange.
    ///
    /// Time: O(k) where k is number of orders matched.
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
        user_id: UserId,
        order_request: OrderRequest,
    ) -> MatcherResult {
        self.check_order(user_id, order_request)?;
        let mut order = self.build_new_order(
            order_request.quantity,
            order_request.price,
            order_request.side,
        );

        let event_id = order_request.market;
        let book = self
            .orderbooks
            .get_mut(&event_id)
            .ok_or(RejectReason::MarketNotFound)?; // infallible

        for fill in book.add(order) {
            self.manager.on_trade(
                user_id,
                self.order_owner[&fill.id].user_id,
                order_request.market,
                fill.quantity,
                fill.price,
                order_request.side,
            );
            order.quantity -= fill.quantity;
            if fill.done {
                self.order_owner.remove(&fill.id);
            }
        }

        let book = self
            .orderbooks
            .get_mut(&order_request.market)
            .ok_or(RejectReason::MarketNotFound)?; // infallible

        let mut quantity = order_request.quantity; // the quantity to report for the event
        if order_request.tif == TimeInForce::IOC {
            quantity = order_request.quantity - order.quantity; // only report the quantity that was filled
            book.remove(order.id);
        } else if order.quantity > 0 {
            self.manager
                .add_resting_order(user_id, order_request.market, order);
            self.order_owner
                .insert(order.id, OrderOwner { user_id, event_id });
        }

        let order = Order::new(order.id, quantity, order.price, order.side);
        let event = book.add_event(timestamp, user_id, order_request.market, order);
        Ok(event)
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
    ) -> MatcherResult {
        let event_id = match self.order_owner.entry(id) {
            Entry::Occupied(entry) if entry.get().user_id == user => entry.remove().event_id,
            _ => return Err(RejectReason::OrderNotFound),
        };

        let book = self
            .orderbooks
            .get_mut(&event_id)
            .ok_or(RejectReason::MarketNotFound)?; // infallible
        let order = book.remove(id).ok_or(RejectReason::OrderNotFound)?; // infallible

        self.manager.remove_order(user, event_id, order);
        let event = book.cancel_event(timestamp, event_id, user, id);
        Ok(event)
    }

    fn check_order(&self, user: UserId, order: OrderRequest) -> Result<(), RejectReason> {
        if order.price == 0 || order.price >= RESOLVE_PRICE {
            Err(RejectReason::InvalidPrice)?;
        }
        if order.quantity == 0 {
            Err(RejectReason::InvalidQuantity)?;
        }
        let Some(book) = self.orderbooks.get(&order.market) else {
            return Err(RejectReason::MarketNotFound);
        };
        if !self
            .manager
            .can_afford(user, order.market, order.quantity, order.price, order.side)
        {
            Err(RejectReason::InsufficientFunds)?;
        }
        let is_marketable = book.is_marketable(order.price, order.side);
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
        MarketUpdate, MarketId, Exchange, OrderBook, OrderRequest, Price, RejectReason, TimeInForce,
        Timestamp, UserId, RESOLVE_PRICE,
    };

    const EVENT: MarketId = 1;
    const TIME: Timestamp = 0;
    const ASK_PRICE: Price = 7000;
    const BID_PRICE: Price = 6000;
    const TAKER: UserId = 1;
    const MAKER: UserId = 2;

    /// Set up exchange with two users with balances and one book.
    fn setup_default_scenario() -> Exchange {
        let mut exch = Exchange::default();
        exch.add_event(TIME, EVENT).unwrap();
        exch.deposit(TAKER, 10 * i64::from(RESOLVE_PRICE));
        exch.deposit(MAKER, 10 * i64::from(RESOLVE_PRICE));
        exch
    }

    #[test]
    fn test_submit_ioc_then_cancel() {
        let mut exch = setup_default_scenario();
        let order = OrderRequest::buy(EVENT, 10, BID_PRICE, TimeInForce::IOC);
        let event = exch.submit_order(TIME, TAKER, order);
        assert_eq!(event, Err(RejectReason::IOCNotMarketable));

        let event = exch.cancel_order(TIME, TAKER, 0);
        assert_eq!(event, Err(RejectReason::OrderNotFound));
    }

    #[test]
    fn test_submit_gtc_then_cancel_twice() {
        let mut exch = setup_default_scenario();
        let order = OrderRequest::buy(EVENT, 10, BID_PRICE, TimeInForce::GTC);
        let event = exch.submit_order(TIME, TAKER, order);
        assert_eq!(
            event,
            Ok(MarketUpdate::buy(TIME, 0, EVENT, TAKER, 0, 10, BID_PRICE))
        );

        assert_eq!(exch.manager.get_balance(TAKER), 100000);
        assert_eq!(exch.manager.get_available(TAKER), 40000);

        let event = exch.cancel_order(TIME, TAKER, 0);
        assert_eq!(event, Ok(MarketUpdate::remove(TIME, 1, EVENT, TAKER, 0)));

        assert_eq!(exch.manager.get_available(TAKER), 100000);

        let event = exch.cancel_order(TIME, TAKER, 0);
        assert_eq!(event, Err(RejectReason::OrderNotFound));
    }

    #[test]
    fn test_cancel_traded_order() {
        let mut exch = setup_default_scenario();

        let order = OrderRequest::sell(EVENT, 1, ASK_PRICE, TimeInForce::GTC);
        let event = exch.submit_order(TIME, MAKER, order);
        assert_eq!(
            event,
            Ok(MarketUpdate::sell(TIME, 0, EVENT, MAKER, 0, 1, ASK_PRICE))
        );

        assert_eq!(exch.manager.get_available(MAKER), 97000);

        let order = OrderRequest::buy(EVENT, 1, ASK_PRICE, TimeInForce::GTC);
        let event = exch.submit_order(TIME, TAKER, order);
        assert_eq!(
            event,
            Ok(MarketUpdate::buy(TIME, 1, EVENT, TAKER, 1, 1, ASK_PRICE))
        );

        assert_eq!(exch.manager.get_balance(MAKER), 97000);
        assert_eq!(exch.manager.get_available(MAKER), 97000);

        let event = exch.cancel_order(TIME, MAKER, 0);
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
        assert_eq!(event, Ok(MarketUpdate::sell(time, 0, book, bob, 0, 3, 4000)));

        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 1, book, bob, 1, 3, 4000)));

        let order = OrderRequest::buy(book, 5, 4000, TimeInForce::GTC);
        assert!(exch.submit_order(time, cat, order).is_ok());

        let event = exch.cancel_order(time, bob, 0);
        assert_eq!(event, Err(RejectReason::OrderNotFound));
    }

    #[test]
    fn test_unmarketable_ioc() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let user = 1;
        let order = OrderRequest::sell(book, 1, 5500, TimeInForce::GTC);
        let event = exch.submit_order(time, user, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 0, book, user, 0, 1, 5500)));

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
        assert_eq!(event, Ok(MarketUpdate::sell(time, 0, book, bob, 0, 5, 4000)));

        assert_eq!(exch.manager.get_available(bob), 70000);

        let order = OrderRequest::buy(book, 2, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 1, book, bob, 1, 2, 4000)));

        assert_eq!(exch.manager.get_balance(bob), 100000);
        assert_eq!(exch.manager.get_available(bob), 82000);

        exch.cancel_order(time, bob, 0);
        assert_eq!(exch.manager.get_available(bob), 100000);
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
        assert_eq!(event, Ok(MarketUpdate::sell(time, 0, book, bob, 0, 3, 4000)));
        let order = OrderRequest::sell(book, 3, 4200, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 1, book, bob, 1, 3, 4200)));
        let order = OrderRequest::sell(book, 3, 4100, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 2, book, bob, 2, 3, 4100)));
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 3, book, bob, 3, 3, 4100)));

        assert_eq!(exch.manager.get_available(bob), 100000 - 70800);
        assert_eq!(exch.manager.get_balance(bob), 100000);

        let order = OrderRequest::buy(book, 7, 4300, TimeInForce::GTC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 4, book, cat, 4, 7, 4300)));

        assert_eq!(exch.manager.get_balance(bob), 58400);
        assert_eq!(exch.manager.get_balance(cat), 71600);
        assert_eq!(exch.manager.get_available(bob), 29200); // 100000 - 70800 + 41600
        assert_eq!(exch.manager.get_available(bob), 29200); // 100000 - 17400 - 11800
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

        assert_eq!(exch.manager.get_position(bob, book), -7);
        assert_eq!(exch.manager.get_position(cat, book), 7);
        assert_eq!(exch.manager.get_balance(bob), 58100);
        assert_eq!(exch.manager.get_balance(cat), 71900);
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
        assert_eq!(event, Ok(MarketUpdate::buy(time, 0, book, bob, 0, 3, 40)));
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 1, book, bob, 1, 3, 40)));
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 2, book, bob, 2, 3, 40)));

        let order = OrderRequest::buy(book, 3, 39, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 3, book, bob, 3, 3, 39)));

        let event = exch.cancel_order(time, bob, 1);
        assert_eq!(event, Ok(MarketUpdate::remove(time, 4, book, bob, 1)));

        let order = OrderRequest::sell(book, 7, 39, TimeInForce::GTC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 5, book, cat, 4, 7, 39)));

        assert_eq!(exch.manager.get_position(bob, book), 7);
        assert_eq!(exch.manager.get_position(cat, book), -7);
        assert_eq!(exch.manager.get_balance(bob), 99721);
        assert_eq!(exch.manager.get_balance(cat), 30279);
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
        assert_eq!(event, Ok(MarketUpdate::sell(time, 0, book, bob, 0, 3, 4000)));

        let order = OrderRequest::buy(book, 5, 4000, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 1, book, cat, 1, 3, 4000)));

        assert_eq!(exch.manager.get_position(bob, book), -3);
        assert_eq!(exch.manager.get_position(cat, book), 3);

        let event = exch.resolve(time, book, 7000);
        assert_eq!(event, Ok(MarketUpdate::resolve(time, 2, book, 0, 7000)));
        assert_eq!(exch.manager.get_balance(bob), 91000);
        assert_eq!(exch.manager.get_balance(cat), 109000);

        let order = OrderRequest::sell(book, 3, 4000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Err(RejectReason::MarketNotFound));
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
        assert_eq!(event, Ok(MarketUpdate::sell(time, 0, book, bob, 0, 10, 5250)));

        // cat submits order
        let order = OrderRequest::buy(book, 1, 5250, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 1, book, cat, 1, 1, 5250)));

        // bob places resting order
        let order = OrderRequest::buy(book, 10, 4750, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 2, book, bob, 2, 10, 4750)));

        // cat submits order
        let order = OrderRequest::sell(book, 1, 4750, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 3, book, cat, 3, 1, 4750)));

        assert_eq!(exch.manager.get_balance(bob), 100500);
        assert_eq!(exch.manager.get_balance(cat), 99500);
    }

    #[test]
    fn trade_with_top_of_book() {
        let mut exch = setup_default_scenario();
        let time = 1;
        let book = 1;
        let bob = 1;
        let cat = 2;

        // bob places resting order
        let order = OrderRequest::sell(book, 5, ASK_PRICE, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(
            event,
            Ok(MarketUpdate::sell(time, 0, book, bob, 0, 5, ASK_PRICE))
        );

        let order = OrderRequest::buy(book, 5, BID_PRICE, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(
            event,
            Ok(MarketUpdate::buy(time, 1, book, bob, 1, 5, BID_PRICE))
        );

        assert_eq!(exch.manager.get_available(bob), 70000);

        let order = OrderRequest::buy(book, 1, 9999, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 2, book, cat, 2, 1, 9999)));
        let order = OrderRequest::sell(book, 1, 1, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 3, book, cat, 3, 1, 1)));

        assert_eq!(exch.manager.get_balance(bob), 101000);
        assert_eq!(exch.manager.get_balance(cat), 99000);
        assert_eq!(exch.manager.get_available(bob), 77000);
        assert_eq!(exch.manager.get_available(cat), 99000);

        assert!(exch.cancel_order(time, bob, 0).is_ok());
        assert!(exch.cancel_order(time, bob, 1).is_ok());

        assert_eq!(exch.manager.get_available(bob), 101000);
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
        assert_eq!(event, Ok(MarketUpdate::sell(time, 0, book, bob, 0, 5, 7000)));

        assert_eq!(exch.manager.get_available(bob), 85000); // 100000 - 15000

        let order = OrderRequest::buy(book, 1, 9999, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 1, book, cat, 1, 1, 9999)));

        assert_eq!(exch.manager.get_available(bob), 85000);
        assert_eq!(exch.manager.get_balance(bob), 97000);
        assert_eq!(exch.manager.get_balance(cat), 93000);
        assert_eq!(exch.manager.get_available(cat), 93000);

        assert!(exch.cancel_order(time, bob, 0).is_ok());
        assert_eq!(exch.manager.get_available(bob), 97000);
        assert_eq!(exch.manager.get_available(cat), 93000);
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
        assert_eq!(event, Ok(MarketUpdate::buy(time, 0, book, bob, 0, 5, 7000)));

        // assert_eq!(exch.tracker.get_available(bob), 65000); // 100000 - 35000

        let order = OrderRequest::sell(book, 1, 1, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 1, book, cat, 1, 1, 1)));

        assert_eq!(exch.manager.get_balance(bob), 93000);
        assert_eq!(exch.manager.get_available(bob), 65000);

        assert_eq!(exch.manager.get_balance(cat), 97000);
        assert_eq!(exch.manager.get_available(cat), 97000);

        assert!(exch.cancel_order(time, bob, 0).is_ok());
        assert_eq!(exch.manager.get_available(bob), 93000);
        assert_eq!(exch.manager.get_available(cat), 97000);
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
        assert_eq!(event, Ok(MarketUpdate::sell(time, 0, book, bob, 0, 5, 6000)));

        let order = OrderRequest::buy(book, 5, 5000, TimeInForce::GTC);
        let event = exch.submit_order(time, bob, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 1, book, bob, 1, 5, 5000)));

        assert_eq!(exch.manager.get_balance(bob), 100000);
        assert_eq!(exch.manager.get_available(bob), 75000);

        let order = OrderRequest::buy(book, 1, 9999, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::buy(time, 2, book, cat, 2, 1, 9999)));

        assert_eq!(exch.manager.get_available(cat), 94000);
        assert_eq!(exch.manager.get_balance(cat), 94000);
        assert_eq!(exch.manager.get_available(bob), 80000);
        assert_eq!(exch.manager.get_balance(bob), 96000);

        let order = OrderRequest::sell(book, 3, 1, TimeInForce::IOC);
        let event = exch.submit_order(time, cat, order);
        assert_eq!(event, Ok(MarketUpdate::sell(time, 3, book, cat, 3, 3, 1)));

        assert_eq!(exch.manager.get_balance(cat), 89000);
        assert_eq!(exch.manager.get_available(cat), 89000);

        assert_eq!(exch.manager.get_balance(bob), 91000);
        assert_eq!(exch.manager.get_available(bob), 81000);

        assert!(exch.cancel_order(time, bob, 0).is_ok());
        assert!(exch.cancel_order(time, bob, 1).is_ok());

        assert_eq!(exch.manager.get_available(bob), 91000);
    }
}
