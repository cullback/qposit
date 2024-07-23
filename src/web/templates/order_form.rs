use askama::Template;
use lobster::MarketId;

#[derive(Template, Debug, Clone)]
#[template(path = "order_form.html")]
pub struct OrderForm {
    market_id: MarketId,
    quantity: String,
    price: String,
    quantity_message: String,
    price_message: String,
    message: String,
}

impl OrderForm {
    pub fn new(market_id: MarketId) -> Self {
        Self {
            market_id,
            quantity: String::new(),
            price: String::new(),
            quantity_message: String::new(),
            price_message: String::new(),
            message: String::new(),
        }
    }
    pub const fn with_messages(
        market_id: MarketId,
        quantity: String,
        price: String,
        quantity_message: String,
        price_message: String,
        message: String,
    ) -> Self {
        Self {
            market_id,
            quantity,
            price,
            quantity_message,
            price_message,
            message,
        }
    }
}
