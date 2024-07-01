//! # Book Service
//!
//! The book service tracks state of all events and streams price-level orderbooks
//! to the front end UI.
//!
//! Every BookUpdate represents a change in the order book state that needs to be
//! broadcast to all clients.
//!
//! - volume
//! - last price
//! - best bid, best ask
//! - order book state
//!
//! TODO: update state more efficiently
//! - track price levels individually instead of updating everything on every event.
use lobster::Price;
use lobster::{Action, Balance, BookUpdate, EventId};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::models;

/// Snapshot of the latest order book data to be rendered.
#[derive(Debug, Clone)]
pub struct EventData {
    pub event_id: EventId,
    pub book: lobster::OrderBook,
    pub best_bid_price: Option<Price>,
    pub best_ask_price: Option<Price>,
    pub last_price: Option<Price>,
    pub volume: Balance,
}

impl EventData {
    pub fn new2(event: &models::event::Event, orderbook: lobster::OrderBook) -> Self {
        Self {
            event_id: event.id,
            best_bid_price: orderbook.best_bid().map(|x| x.price),
            best_ask_price: orderbook.best_ask().map(|x| x.price),
            book: orderbook,
            last_price: event.last_trade_price,
            volume: event.volume,
        }
    }

    pub fn new_default(event_id: EventId) -> Self {
        Self {
            event_id,
            book: lobster::OrderBook::default(),
            best_bid_price: None,
            best_ask_price: None,
            last_price: None,
            volume: 0,
        }
    }

    pub fn on_event(&mut self, event: BookUpdate) {
        match event.action {
            Action::Add(order) => {
                let fills = self.book.add(order);
                for fill in fills {
                    self.volume += Balance::from(fill.quantity) * Balance::from(fill.price);
                    self.last_price = Some(fill.price);
                }
                self.best_bid_price = self.book.best_bid().map(|x| x.price);
                self.best_ask_price = self.book.best_ask().map(|x| x.price);
            }
            Action::Remove { id } => {
                assert!(self.book.remove(id).is_some());
                self.best_bid_price = self.book.best_bid().map(|x| x.price);
                self.best_ask_price = self.book.best_ask().map(|x| x.price);
            }
            Action::Resolve { price } => {
                self.last_price = Some(price);
            }
            Action::AddEvent => todo!(),
        }
    }
}

struct EventService {
    events: HashMap<EventId, EventData>,
}

impl EventService {
    pub async fn new(db: &SqlitePool) -> Self {
        let mut events = HashMap::new();
        for event in models::event::Event::get_active(db).await.unwrap() {
            let event_id = event.id;
            let orderbook = models::order::Order::build_orderbook(db, event.id)
                .await
                .unwrap();
            let book_data = EventData::new2(&event, orderbook);
            events.insert(event_id, book_data);
        }

        Self { events }
    }

    fn on_event(&mut self, event: BookUpdate) -> EventData {
        if matches!(event.action, Action::AddEvent) {
            self.events
                .insert(event.book, EventData::new_default(event.book));
        }
        let book = self.events.get_mut(&event.book).unwrap();
        book.on_event(event);
        book.clone()
    }
}

pub fn start_book_service(
    db: SqlitePool,
    mut feed: broadcast::Receiver<BookUpdate>,
    book_stream: broadcast::Sender<EventData>,
) {
    tokio::spawn({
        async move {
            info!("Starting book service...");
            let mut state = EventService::new(&db).await;

            while let Ok(event) = feed.recv().await {
                let orderbook = state.on_event(event);
                book_stream.send(orderbook).unwrap();
            }
        }
    });
}
