use askama::Template;
use lobster::EventId;

use crate::models::{event::Event, market::Market};

use super::{format_price_to_string, mid_to_string};

#[allow(dead_code)]
struct BookView {
    id: EventId,
    market_id: i64,
    title: String,
    last_trade_price: String,
    mid_price: String,
}

impl From<Event> for BookView {
    fn from(book: Event) -> Self {
        Self {
            id: book.id,
            market_id: book.market_id,
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
    markets: Vec<(Market, Vec<BookView>)>,
}

impl HomePage {
    pub fn new(username: String, markets: Vec<(Market, Vec<Event>)>) -> Self {
        Self {
            username,
            markets: markets
                .into_iter()
                .map(|(market, events)| {
                    let events = events.into_iter().map(BookView::from).collect();
                    (market, events)
                })
                .collect(),
        }
    }
}
