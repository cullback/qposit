use askama::Template;
use lobster::MarketId;

use crate::models::{event::Event, market::Market};

use super::display_price;

#[allow(dead_code)]
struct BookView {
    id: MarketId,
    event_id: i64,
    title: String,
    display_price: String,
    is_resolved: bool,
}

impl From<Market> for BookView {
    fn from(market: Market) -> Self {
        Self {
            id: market.id,
            event_id: market.event_id,
            title: market.title,
            display_price: display_price(
                market.best_bid_price,
                market.best_ask_price,
                market.last_trade_price,
                market.outcome,
            ),
            is_resolved: market.outcome.is_some(),
        }
    }
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomePage {
    /// Username for header. Empty string if not logged in.
    username: String,
    events: Vec<(Event, Vec<BookView>)>,
}

impl HomePage {
    pub fn new(username: String, events: Vec<(Event, Vec<Market>)>) -> Self {
        Self {
            username,
            events: events
                .into_iter()
                .map(|(event, markets)| {
                    let markets = markets.into_iter().map(BookView::from).collect();
                    (event, markets)
                })
                .collect(),
        }
    }
}
