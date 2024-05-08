use askama::Template;

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: String,
    pub quantity: String,
    pub value: String,
}

#[derive(Template, Debug, Clone)]
#[template(path = "orderbook.html")]
pub struct OrderBook {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

impl OrderBook {
    pub fn new(bids: Vec<PriceLevel>, asks: Vec<PriceLevel>) -> Self {
        Self { bids, asks }
    }
}
