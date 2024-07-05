use askama::Template;
use lobster::MarketId;

use crate::models::{event::Event, market::Market};

use super::{format_price_to_string, mid_to_string};

#[allow(dead_code)]
struct BookView {
    id: MarketId,
    event_id: i64,
    title: String,
    last_trade_price: String,
    mid_price: String,
}

impl From<Market> for BookView {
    fn from(book: Market) -> Self {
        Self {
            id: book.id,
            event_id: book.event_id,
            title: book.title,
            last_trade_price: format_price_to_string(book.last_trade_price.unwrap_or(0)),
            mid_price: mid_to_string(book.best_bid_price, book.best_ask_price),
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
