use super::math::trade_cost;
use super::{book_portfolio::BookPortfolio, user_portfolio::UserPortfolio};
use crate::{Balance, MarketId, Order, Position, Price, Quantity, Side, UserId, RESOLVE_PRICE};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PortfolioManager {
    users: HashMap<UserId, UserPortfolio>,
}

impl PortfolioManager {
    /// Constructs a new balance tracker from an initial state.
    #[must_use]
    pub fn new(
        balances: &HashMap<UserId, Balance>,
        positions: &HashMap<(UserId, MarketId), Position>,
    ) -> Self {
        let mut users: HashMap<UserId, UserPortfolio> = HashMap::new();

        for (&user_id, &balance) in balances {
            let user = users.entry(user_id).or_default();
            user.add_balance(balance);

            for (&(user_id2, book), &position) in positions {
                if user_id == user_id2 {
                    user.perbook
                        .insert(book, BookPortfolio::with_position(position));
                }
            }
        }
        Self { users }
    }

    /// Deposits an amount into a user's account. Creates the user if they don't exist.
    pub fn deposit(&mut self, user: UserId, amount: Balance) {
        let user = self.users.entry(user).or_default();
        user.add_balance(amount);
    }

    /// Returns `true` if placing an order with these arguments would not exceed
    /// the user's available.
    #[must_use]
    pub fn can_afford(
        &self,
        user: UserId,
        book: MarketId,
        quantity: Quantity,
        price: Price,
        side: Side,
    ) -> bool {
        let Some(user) = self.users.get(&user) else {
            return false;
        };
        user.can_afford(book, quantity, price, side)
    }

    /// Adds exposure for a resting order to the tracker.
    ///
    /// # Panics
    ///
    /// Panics if the user does not exist or cannot afford the order.
    pub fn add_resting_order(&mut self, user: UserId, book: MarketId, order: Order) {
        let user = self.users.get_mut(&user).expect("Invariant");
        assert!(
            user.can_afford(book, order.quantity, order.price, order.side),
            "Invariant"
        );
        let perbook = user.perbook.entry(book).or_default();
        perbook.add_exposure(order);
        user.available -= perbook.compute_change();
        assert!(user.available >= 0, "Invariant");
    }

    /// Removes exposure of cancelled resting order from the tracker.
    ///
    /// # Panics
    ///
    /// Panics if the user does not exist or the user does not have the order.
    pub fn remove_order(&mut self, user: UserId, book: MarketId, order: Order) {
        let user = self.users.get_mut(&user).expect("Invariant");
        let book = user.perbook.get_mut(&book).expect("Invariant");
        book.remove_exposure(order.quantity, order.price, order.side);
        user.available -= book.compute_change();
    }

    /// Updates the tracker with a trade event.
    ///
    /// # Panics
    ///
    /// Panics if the taker or maker don't exist, or if it causes either users
    /// available to go negative.
    pub fn on_trade(
        &mut self,
        taker: UserId,
        maker: UserId,
        book: MarketId,
        quantity: Quantity,
        price: Price,
        side: Side,
    ) {
        #[allow(clippy::cast_possible_wrap, clippy::as_conversions)]
        let signed_quantity = match side {
            Side::Buy => quantity as i32,
            Side::Sell => -(quantity as i32),
        };

        let taker = self.users.get_mut(&taker).expect("Invariant");
        let perbook = taker.perbook.entry(book).or_default();
        let cost = trade_cost(perbook.position, quantity, price, side);
        perbook.position += signed_quantity;
        taker.add_balance(-cost);

        let maker = self.users.get_mut(&maker).expect("Invariant");
        let perbook = maker.perbook.entry(book).or_default();
        let cost = trade_cost(perbook.position, quantity, price, !side);
        perbook.position -= signed_quantity;
        perbook.remove_exposure(quantity, price, !side);

        maker.available -= perbook.compute_change();
        maker.add_balance(-cost);
    }

    /// Resolves a book to a specific price. Zeroes out the position and adds winnings
    /// to users balance.
    pub fn resolve(&mut self, book: MarketId, price: Price) -> Vec<UserId> {
        let mut winners = Vec::new();
        for (&user_id, user) in self.users.iter_mut() {
            let Some(book) = user.perbook.remove(&book) else {
                continue;
            };
            user.available += book.last_exposure;

            if book.position == 0 {
                continue;
            }
            let position_value = if book.position >= 0 {
                Balance::from(price) * Balance::from(book.position)
            } else {
                Balance::from(RESOLVE_PRICE - price) * -Balance::from(book.position)
            };
            user.add_balance(position_value);
            winners.push(user_id);
        }
        winners
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn get_balance(&self, user: UserId) -> Balance {
        self.users.get(&user).map(|x| x.balance).unwrap_or_default()
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn get_available(&self, user: UserId) -> Balance {
        self.users
            .get(&user)
            .map(|x| x.available)
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn get_position(&self, user: UserId, book: MarketId) -> Position {
        self.users
            .get(&user)
            .and_then(|x| x.perbook.get(&book))
            .map(|x| x.position)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASK_PRICE: Price = 7000;
    const BID_PRICE: Price = 6000;
    const BOOK: MarketId = 1;

    const MAKER: UserId = 1;
    const TAKER: UserId = 2;

    #[test]
    fn test_deposit() {
        let mut manager = PortfolioManager::default();
        manager.deposit(MAKER, 100000);

        let user = manager.users.get(&MAKER).unwrap();
        assert_eq!(user.balance, 100000);
        assert_eq!(user.available, 100000);
    }

    #[test]
    fn test1() {
        let mut manager = PortfolioManager::default();
        manager.deposit(MAKER, 100000);
        manager.add_resting_order(MAKER, BOOK, Order::sell(0, 5, ASK_PRICE));
        manager.add_resting_order(MAKER, BOOK, Order::buy(0, 5, BID_PRICE));

        assert_eq!(manager.users[&MAKER].available, 70000);
    }

    #[test]
    fn test_quote_buy_sell_even_more() {
        let mut manager = PortfolioManager::default();
        manager.deposit(TAKER, 100000);
        manager.deposit(MAKER, 100000);

        manager.add_resting_order(MAKER, BOOK, Order::sell(0, 5, ASK_PRICE));
        manager.add_resting_order(MAKER, BOOK, Order::buy(0, 5, BID_PRICE));

        assert_eq!(manager.users[&MAKER].available, 70000);

        manager.on_trade(TAKER, MAKER, BOOK, 1, ASK_PRICE, Side::Buy);

        assert_eq!(manager.users[&MAKER].balance, 97000);
        assert_eq!(manager.users[&MAKER].available, 77000);

        manager.on_trade(TAKER, MAKER, BOOK, 3, BID_PRICE, Side::Sell);

        assert_eq!(manager.users[&MAKER].balance, 89000);
        assert_eq!(manager.users[&MAKER].available, 77000);

        manager.remove_order(MAKER, BOOK, Order::sell(0, 4, ASK_PRICE));
        manager.remove_order(MAKER, BOOK, Order::buy(0, 2, BID_PRICE));

        assert_eq!(manager.users[&MAKER].balance, 89000);
        assert_eq!(manager.users[&MAKER].available, 89000);
    }

    #[test]
    fn test_quote_sell_buy_even_more() {
        let mut manager = PortfolioManager::default();
        manager.deposit(TAKER, 100000);
        manager.deposit(MAKER, 100000);

        manager.add_resting_order(MAKER, BOOK, Order::sell(0, 5, ASK_PRICE));
        manager.add_resting_order(MAKER, BOOK, Order::buy(0, 5, BID_PRICE));

        assert_eq!(manager.users[&MAKER].available, 70000);

        manager.on_trade(TAKER, MAKER, BOOK, 1, BID_PRICE, Side::Sell);

        assert_eq!(manager.users[&MAKER].balance, 94000);
        assert_eq!(manager.users[&MAKER].available, 70000);

        manager.on_trade(TAKER, MAKER, BOOK, 3, ASK_PRICE, Side::Buy);

        assert_eq!(manager.users[&MAKER].balance, 95000);
        assert_eq!(manager.users[&MAKER].available, 89000);

        manager.remove_order(MAKER, BOOK, Order::sell(0, 2, ASK_PRICE));
        manager.remove_order(MAKER, BOOK, Order::buy(0, 4, BID_PRICE));

        assert_eq!(manager.users[&MAKER].available, 95000);
    }

    #[test]
    fn test_from_wei() {
        let balances = HashMap::from([(MAKER, 100000)]);
        let positions = HashMap::from([((MAKER, BOOK), 0)]);
        let mut manager = PortfolioManager::new(&balances, &positions);

        manager.add_resting_order(MAKER, BOOK, Order::sell(0, 2, 100));
        manager.add_resting_order(MAKER, BOOK, Order::sell(0, 2, 200));
        manager.add_resting_order(MAKER, BOOK, Order::sell(0, 2, 500));
        manager.add_resting_order(MAKER, BOOK, Order::sell(0, 2, 600));

        manager.users.get_mut(&MAKER).unwrap().available = 52800;

        manager.remove_order(MAKER, BOOK, Order::sell(0, 2, 200));

        manager.users.get_mut(&MAKER).unwrap().available = 72400;

        assert_eq!(manager.users[&MAKER].available, 72400);
    }

    #[test]
    fn test_resolve() {
        let mut manager = PortfolioManager::default();
        manager.deposit(TAKER, 100000);
        manager.deposit(MAKER, 100000);

        manager.add_resting_order(MAKER, BOOK, Order::sell(0, 5, ASK_PRICE));

        assert_eq!(manager.users[&MAKER].available, 85000);

        manager.on_trade(TAKER, MAKER, BOOK, 2, ASK_PRICE, Side::Buy);

        manager.resolve(BOOK, RESOLVE_PRICE);

        assert_eq!(manager.get_balance(MAKER), 94000);
        assert_eq!(manager.get_available(MAKER), 94000);
        assert_eq!(manager.get_position(MAKER, BOOK), 0);

        assert_eq!(manager.get_balance(TAKER), 106000);
        assert_eq!(manager.get_available(TAKER), 106000);
        assert_eq!(manager.get_position(TAKER, BOOK), 0);
    }
}
