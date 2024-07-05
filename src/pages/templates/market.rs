use super::order_form::OrderForm;
use crate::{actors::book_service::MarketData, models::market::Market, pages::MarketUpdate};
use askama::Template;

#[derive(Template, Debug, Clone)]
#[template(path = "market.html")]
pub struct BookHtml {
    pub title: String,
    pub update: MarketUpdate,
    pub order_form: OrderForm,
}

impl BookHtml {
    pub fn new(market: Market, market_data: &MarketData) -> Self {
        Self {
            title: market.title,
            update: MarketUpdate::from(market_data),
            order_form: OrderForm::new(market_data.market_id),
        }
    }
}
