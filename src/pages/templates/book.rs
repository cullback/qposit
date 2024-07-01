use super::order_form::OrderForm;
use crate::{actors::book_service::EventData, models::event::Event, pages::OrderBook};
use askama::Template;

#[derive(Template, Debug, Clone)]
#[template(path = "book.html")]
pub struct BookHtml {
    pub title: String,
    pub book_data: OrderBook,
    pub order_form: OrderForm,
}

impl BookHtml {
    pub fn new(book: Event, book_data: &EventData) -> Self {
        Self {
            title: book.title,
            book_data: OrderBook::from(book_data),
            order_form: OrderForm::new(book_data.event_id),
        }
    }
}
