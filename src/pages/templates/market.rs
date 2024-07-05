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
    pub fn new(book: Market, book_data: &MarketData) -> Self {
        Self {
            title: book.title,
            update: MarketUpdate::from(book_data),
            order_form: OrderForm::new(book_data.market_id),
        }
    }
}
