mod fill;
mod order;
mod side;

use std::cmp::Reverse;

pub use self::{fill::Fill, order::Order, side::Side};

/// Globally unique order id.
pub type OrderId = i64;

/// Represents a number of contracts.
pub type Quantity = u32;

/// Price in basis points.
pub type Price = u16;

#[derive(Default, Debug)]
pub struct OrderBook {
    /// Bids, sorted by price ascending
    bids: Vec<Order>,
    /// Asks, sorted by price descending
    asks: Vec<Order>,
}

impl OrderBook {
    /// Returns the number of open orders in the book.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bids.len() + self.asks.len()
    }

    /// Returns `true` if the book contains no open orders.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the bids from best to worst.
    pub fn bids(&self) -> impl Iterator<Item = Order> + '_ {
        self.bids.iter().rev().copied()
    }

    /// Returns an iterator over the asks from best to worst.
    pub fn asks(&self) -> impl Iterator<Item = Order> + '_ {
        self.asks.iter().rev().copied()
    }

    /// Returns the best bid.
    #[must_use]
    pub fn best_bid(&self) -> Option<Order> {
        self.bids().next()
    }

    /// Returns the best ask.
    #[must_use]
    pub fn best_ask(&self) -> Option<Order> {
        self.asks().next()
    }

    /// Adds an order to the order book. Returns a list of fills if the order was marketable.
    pub fn add(&mut self, order: Order) -> Vec<Fill> {
        match order.side {
            Side::Buy => self.buy(order.id, order.quantity, order.price),
            Side::Sell => self.sell(order.id, order.quantity, order.price),
        }
    }

    /// Removes an order by id.
    pub fn remove(&mut self, id: OrderId) -> Option<Order> {
        if let Some(i) = self.bids.iter().position(|order| order.id == id) {
            return Some(self.bids.remove(i));
        }
        if let Some(i) = self.asks.iter().position(|order| order.id == id) {
            return Some(self.asks.remove(i));
        }
        None
    }

    fn buy(&mut self, id: OrderId, mut quantity: Quantity, price: Price) -> Vec<Fill> {
        let mut fills = Vec::new();
        let mut i = 0;
        for order in self
            .asks
            .iter_mut()
            .rev()
            .take_while(|order| order.price <= price)
        {
            if quantity >= order.quantity {
                fills.push(Fill::new(order.id, order.quantity, order.price, true));
                quantity -= order.quantity;
                i += 1;
            } else {
                fills.push(Fill::new(order.id, quantity, order.price, false));
                order.quantity -= quantity;
                quantity = 0;
                break;
            }
        }
        self.asks.drain(self.asks.len() - i..);
        if quantity > 0 {
            self.bids.insert(
                0,
                Order {
                    id,
                    quantity,
                    price,
                    side: Side::Buy,
                },
            );
            self.bids.sort_by_key(|order| order.price);
        }
        fills
    }

    fn sell(&mut self, id: OrderId, mut quantity: Quantity, price: Price) -> Vec<Fill> {
        let mut fills = Vec::new();
        let mut i = 0;
        for order in self
            .bids
            .iter_mut()
            .rev()
            .take_while(|order| order.price >= price)
        {
            if quantity >= order.quantity {
                fills.push(Fill::new(order.id, order.quantity, order.price, true));
                quantity -= order.quantity;
                i += 1;
            } else {
                fills.push(Fill::new(order.id, quantity, order.price, false));
                order.quantity -= quantity;
                quantity = 0;
                break;
            }
        }
        self.bids.drain(self.bids.len() - i..);
        if quantity > 0 {
            self.asks.insert(
                0,
                Order {
                    id,
                    quantity,
                    price,
                    side: Side::Sell,
                },
            );
            self.asks.sort_by_key(|order| Reverse(order.price));
        }
        fills
    }
}

#[cfg(test)]
mod tests {
    use super::{Fill, Order, OrderBook, Price, Quantity, Side};

    #[test]
    fn add_then_remove() {
        let mut book = lobster::default();
        let id = 1;
        book.add(Order::buy(id, 1, 2));
        assert_eq!(book.len(), 1);
        assert!(book.remove(id).is_some());
        assert_eq!(book.len(), 0);
        assert!(book.remove(id).is_none());
    }

    #[test]
    fn multiple_fills_with_cancel() {
        let mut book = lobster::default();
        book.add(Order::sell(0, 2, 5));
        book.add(Order::sell(1, 3, 6));
        book.add(Order::sell(2, 4, 7));
        book.remove(0);
        let fills = book.add(Order::buy(3, 6, 6));
        assert_eq!(fills, vec![Fill::new(1, 3, 6, true)])
    }

    #[test]
    fn fire_for_order_that_was_filled_exactly() {
        let mut book = lobster::default();
        book.add(Order::sell(0, 2, 23));
        let fills = book.add(Order::buy(1, 2, 23));
        assert_eq!(fills, vec![Fill::new(0, 2, 23, true)]);
        let fills = book.add(Order::buy(2, 2, 23));
        assert_eq!(fills, vec![]);

        let mut book = lobster::default();
        book.add(Order::buy(0, 2, 23));
        let fills = book.add(Order::sell(1, 2, 23));
        assert_eq!(fills, vec![Fill::new(0, 2, 23, true)]);
        let fills = book.add(Order::sell(2, 2, 23));
        assert_eq!(fills, vec![]);
    }

    #[test]
    fn fire_for_order_that_was_filled_excessively() {
        let mut book = lobster::default();
        book.add(Order::sell(0, 1, 23));
        let fills = book.add(Order::buy(1, 2, 23));
        assert_eq!(fills, vec![Fill::new(0, 1, 23, true)]);
        let fills = book.add(Order::buy(2, 1, 23));
        assert_eq!(fills, vec![]);

        let mut book = lobster::default();
        book.add(Order::buy(0, 1, 23));
        let fills = book.add(Order::sell(1, 2, 23));
        assert_eq!(fills, vec![Fill::new(0, 1, 23, true)]);
        let fills = book.add(Order::sell(2, 1, 23));
        assert_eq!(fills, vec![]);
    }

    #[test]
    fn trade_twice_with_resting_order() {
        let mut book = lobster::default();
        book.add(Order::sell(0, 2, 23));
        let fills = book.add(Order::buy(1, 1, 23));
        assert_eq!(fills, vec![Fill::new(0, 1, 23, false)]);
        let fills = book.add(Order::buy(2, 1, 23));
        assert_eq!(fills, vec![Fill::new(0, 1, 23, true)]);

        let mut book = lobster::default();
        book.add(Order::buy(0, 2, 23));
        let fills = book.add(Order::sell(1, 1, 23));
        assert_eq!(fills, vec![Fill::new(0, 1, 23, false)]);
        let fills = book.add(Order::sell(2, 1, 23));
        assert_eq!(fills, vec![Fill::new(0, 1, 23, true)]);
    }

    #[test]
    fn test_quantity_limits() {
        let mut book = lobster::default();
        book.add(Order::sell(0, Quantity::MAX, 23));
        let fills = book.add(Order::buy(1, Quantity::MAX, 23));
        assert_eq!(fills, vec![Fill::new(0, Quantity::MAX, 23, true)]);

        let mut book = lobster::default();
        book.add(Order::buy(0, Quantity::MAX, 23));
        let fills = book.add(Order::sell(1, Quantity::MAX, 23));
        assert_eq!(fills, vec![Fill::new(0, Quantity::MAX, 23, true)]);
    }

    #[test]
    fn trade_twice_with_resting_order_price_limits() {
        let mut book = lobster::default();
        book.add(Order::sell(0, 2, Price::MIN));
        let fills = book.add(Order::buy(1, 1, Price::MIN));
        assert_eq!(fills, vec![Fill::new(0, 1, Price::MIN, false)]);
        let fills = book.add(Order::buy(2, 1, Price::MIN));
        assert_eq!(fills, vec![Fill::new(0, 1, Price::MIN, true)]);

        let mut book = lobster::default();
        book.add(Order::buy(0, 2, Price::MIN));
        let fills = book.add(Order::sell(1, 1, Price::MIN));
        assert_eq!(fills, vec![Fill::new(0, 1, Price::MIN, false)]);
        let fills = book.add(Order::sell(2, 1, Price::MIN));
        assert_eq!(fills, vec![Fill::new(0, 1, Price::MIN, true)]);

        let mut book = lobster::default();
        book.add(Order::sell(0, 2, Price::MAX));
        let fills = book.add(Order::buy(1, 1, Price::MAX));
        assert_eq!(fills, vec![Fill::new(0, 1, Price::MAX, false)]);
        let fills = book.add(Order::buy(2, 1, Price::MAX));
        assert_eq!(fills, vec![Fill::new(0, 1, Price::MAX, true)]);

        let mut book = lobster::default();
        book.add(Order::buy(0, 2, Price::MAX));
        let fills = book.add(Order::sell(1, 1, Price::MAX));
        assert_eq!(fills, vec![Fill::new(0, 1, Price::MAX, false)]);
        let fills = book.add(Order::sell(2, 1, Price::MAX));
        assert_eq!(fills, vec![Fill::new(0, 1, Price::MAX, true)]);
    }

    #[test]
    fn test_queue_priority() {
        let mut book = lobster::default();
        book.add(Order::sell(0, 1, 23));
        book.add(Order::sell(1, 1, 23));
        book.add(Order::sell(2, 1, 23));
        let fills = book.add(Order::buy(3, 3, 23));
        assert_eq!(
            fills,
            vec![
                Fill::new(0, 1, 23, true),
                Fill::new(1, 1, 23, true),
                Fill::new(2, 1, 23, true)
            ]
        );
    }
}
