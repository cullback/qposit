use askama::Template;
use exchange::BookId;

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: String,
    pub quantity: String,
    pub value: String,
}

#[derive(Template, Debug, Clone)]
#[template(path = "orderbook.html")]
pub struct OrderBook {
    pub book_id: BookId,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

impl OrderBook {
    pub fn new(book_id: BookId, bids: Vec<PriceLevel>, asks: Vec<PriceLevel>) -> Self {
        Self {
            book_id,
            bids,
            asks,
        }
    }
}
