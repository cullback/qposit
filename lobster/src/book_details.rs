use crate::{
    Fill, Order, OrderBook, OrderId, Price, Side, Tick,
};

#[derive(Debug, Default)]
pub struct BookDetails {
    next_tick: Tick,
    inner: OrderBook,
}

impl BookDetails {
    pub fn get_next_tick(&mut self) -> Tick {
        let tick = self.next_tick;
        self.next_tick = self.next_tick.wrapping_add(1);
        tick
    }

    /// Returns `true` if the price is marketable.
    pub fn is_marketable(&self, price: Price, side: Side) -> bool {
        match side {
            Side::Buy => self.inner.best_ask().is_some_and(|ask| price >= ask.price),
            Side::Sell => self.inner.best_bid().is_some_and(|bid| price <= bid.price),
        }
    }

    pub fn add(&mut self, order: Order) -> Vec<Fill> {
        self.inner.add(order)
    }

    pub fn remove(&mut self, id: OrderId) -> Option<Order> {
        self.inner.remove(id)
    }
}
