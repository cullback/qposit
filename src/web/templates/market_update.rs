use askama::Template;
use lobster::MarketId;

use crate::services::book_service::MarketData;

use super::orderbook::OrderBook;
use super::{display_price, format_balance_to_dollars};

/// Book data delivered over websocket feed.
/// Needs to be applied against an already rendered book.
#[derive(Template, Debug, Clone)]
#[template(path = "market_update.html")]
pub struct MarketUpdate {
    pub market_id: MarketId,
    pub display_price: String,
    pub volume: String,
    pub orderbook: OrderBook,
}

impl From<&MarketData> for MarketUpdate {
    fn from(market: &MarketData) -> Self {
        let volume = format_balance_to_dollars(market.volume);
        let display_price = display_price(
            market.best_bid,
            market.best_ask,
            market.last_price,
            market.outcome,
        );
        Self {
            market_id: market.market_id,
            display_price,
            volume,
            orderbook: OrderBook::new(market.market_id, &market.book),
        }
    }
}
