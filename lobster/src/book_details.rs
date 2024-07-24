use crate::{
    Action, MarketUpdate, MarketId, Fill, Order, OrderBook, OrderId, Price, Side, Tick, Timestamp,
    UserId,
};

#[derive(Debug, Default)]
pub struct BookDetails {
    next_tick: Tick,
    inner: OrderBook,
}

impl BookDetails {
    fn get_next_tick(&mut self) -> Tick {
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

    pub fn add_event(
        &mut self,
        time: Timestamp,
        user: UserId,
        book: MarketId,
        order: Order,
    ) -> MarketUpdate {
        MarketUpdate {
            time,
            tick: self.get_next_tick(),
            book,
            user,
            action: Action::Add(order),
        }
    }
    pub fn cancel_event(
        &mut self,
        time: Timestamp,
        book: MarketId,
        user: UserId,
        id: OrderId,
    ) -> MarketUpdate {
        MarketUpdate::remove(time, self.get_next_tick(), book, user, id)
    }

    pub fn resolve_event(&mut self, time: Timestamp, book: MarketId, price: Price) -> MarketUpdate {
        MarketUpdate::resolve(time, self.get_next_tick(), book, 0, price)
    }
}
