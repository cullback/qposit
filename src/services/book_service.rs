//! # Book Service
//!
//! The book service tracks state of all markets and streams price-level orderbooks
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
//! - track price levels individually instead of updating everything on every market.
use lobster::Price;
use lobster::{Balance, MarketId, MarketUpdate};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::models;

/// Snapshot of the latest order book data to be rendered.
#[derive(Debug, Clone)]
pub struct MarketData {
    pub market_id: MarketId,
    pub book: lobster::OrderBook,
    pub best_bid: Option<Price>,
    pub best_ask: Option<Price>,
    pub last_price: Option<Price>,
    pub outcome: Option<Price>,
    pub volume: Balance,
}

impl MarketData {
    pub fn new(market: &models::market::Market, orderbook: lobster::OrderBook) -> Self {
        Self {
            market_id: market.id,
            best_bid: orderbook.best_bid().map(|x| x.price),
            best_ask: orderbook.best_ask().map(|x| x.price),
            book: orderbook,
            last_price: market.last_price,
            outcome: market.outcome,
            volume: market.volume,
        }
    }

    pub fn new_default(market_id: MarketId) -> Self {
        Self {
            market_id,
            book: lobster::OrderBook::default(),
            best_bid: None,
            best_ask: None,
            last_price: None,
            outcome: None,
            volume: 0,
        }
    }

    fn add_order(&mut self, order: lobster::Order) {
        let fills = self.book.add(order);
        for fill in fills {
            self.volume += Balance::from(fill.quantity) * Balance::from(fill.price);
            self.last_price = Some(fill.price);
        }
        self.best_bid = self.book.best_bid().map(|x| x.price);
        self.best_ask = self.book.best_ask().map(|x| x.price);
    }

    fn remove_order(&mut self, id: lobster::OrderId) {
        assert!(self.book.remove(id).is_some());
        self.best_bid = self.book.best_bid().map(|x| x.price);
        self.best_ask = self.book.best_ask().map(|x| x.price);
    }

    fn resolve(&mut self, price: lobster::Price) {
        self.outcome = Some(price);
    }
}

struct MarketDataService {
    markets: HashMap<MarketId, MarketData>,
}

impl MarketDataService {
    pub async fn new(db: &SqlitePool) -> Self {
        let mut markets = HashMap::new();
        for market in models::market::Market::get_active(db).await.unwrap() {
            let market_id = market.id;
            let orderbook = models::order::Order::build_orderbook(db, market.id)
                .await
                .unwrap();
            let book_data = MarketData::new(&market, orderbook);
            markets.insert(market_id, book_data);
        }

        Self { markets }
    }

    fn on_event(&mut self, update: MarketUpdate) -> Option<MarketData> {
        match update {
            MarketUpdate::AddOrder { market, order, .. } => {
                let market = self.markets.get_mut(&market).unwrap();
                market.add_order(order);
                return Some(market.clone());
            }
            MarketUpdate::RemoveOrder { market, id, .. } => {
                let market = self.markets.get_mut(&market).unwrap();
                market.remove_order(id);
                return Some(market.clone());
            }
            MarketUpdate::ResolveMarket { market, price, .. } => {
                let market = self.markets.get_mut(&market).unwrap();
                market.resolve(price);
                return Some(market.clone());
            }
            MarketUpdate::AddMarket { market, .. } => {
                let market_data = MarketData::new_default(market);
                self.markets.insert(market, market_data.clone());
                return Some(market_data);
            }
            MarketUpdate::Deposit { .. } => {
                return None;
            }
        }
    }
}

pub fn start_book_service(
    db: SqlitePool,
    mut feed: broadcast::Receiver<MarketUpdate>,
    book_stream: broadcast::Sender<MarketData>,
) {
    tokio::spawn({
        async move {
            info!("Starting book service...");
            let mut state = MarketDataService::new(&db).await;

            while let Ok(market) = feed.recv().await {
                let Some(market_data) = state.on_event(market) else {
                    continue;
                };
                book_stream.send(market_data).unwrap();
            }
        }
    });
}
