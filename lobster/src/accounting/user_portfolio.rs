use super::book_portfolio::BookPortfolio;
use super::math::trade_cost;
use crate::{Balance, MarketId, Price, Quantity, Side};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct UserPortfolio {
    pub balance: Balance,
    pub available: Balance,
    pub perbook: HashMap<MarketId, BookPortfolio>,
}

impl UserPortfolio {
    pub fn add_balance(&mut self, amount: Balance) {
        self.balance += amount;
        self.available += amount;
        assert!(self.available >= 0, "Invariant");
        assert!(self.balance >= 0, "Invariant");
    }

    pub fn can_afford(&self, book: MarketId, quantity: Quantity, price: Price, side: Side) -> bool {
        let position = self
            .perbook
            .get(&book)
            .map(|x| x.position)
            .unwrap_or_default();
        let cost = trade_cost(position, quantity, price, side);
        self.available >= cost
    }
}
