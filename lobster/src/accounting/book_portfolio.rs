use super::math::{contracts_combined, contracts_created};

use crate::{Balance, Order, Position, Price, Quantity, Side, RESOLVE_PRICE};

#[derive(Debug, Default)]
pub struct BookPortfolio {
    /// The last exposure computed that adjusted available.
    /// Should be >= 0
    pub last_exposure: Balance,
    pub position: Position,
    bid_value: Balance,
    ask_value: Balance,
    bid_quantity: Quantity,
    ask_quantity: Quantity,
}

impl BookPortfolio {
    pub fn with_position(position: Position) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    pub fn add_exposure(&mut self, order: Order) {
        match order.side {
            Side::Buy => {
                self.bid_quantity += order.quantity;
                self.bid_value += Balance::from(order.quantity) * Balance::from(order.price);
            }
            Side::Sell => {
                self.ask_quantity += order.quantity;
                self.ask_value += Balance::from(order.quantity) * Balance::from(order.price);
            }
        }
    }

    pub fn remove_exposure(&mut self, quantity: Quantity, price: Price, side: Side) {
        match side {
            Side::Buy => {
                self.bid_quantity -= quantity;
                self.bid_value -= Balance::from(quantity) * Balance::from(price);
            }
            Side::Sell => {
                self.ask_quantity -= quantity;
                self.ask_value -= Balance::from(quantity) * Balance::from(price);
            }
        }
    }

    fn compute_exposure(&mut self) -> Balance {
        let created = contracts_created(self.position, self.ask_quantity);
        let ask_exposure =
            Balance::from(created) * Balance::from(RESOLVE_PRICE) - Balance::from(self.ask_value);

        let combined = contracts_combined(self.position, self.bid_quantity);
        let bid_exposure =
            Balance::from(self.bid_value) - Balance::from(combined) * Balance::from(RESOLVE_PRICE);

        ask_exposure.max(bid_exposure)
    }

    pub fn compute_change(&mut self) -> Balance {
        let exposure = self.compute_exposure();
        let change = exposure - self.last_exposure;
        self.last_exposure = exposure;
        change
    }
}
