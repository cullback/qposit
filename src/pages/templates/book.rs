use super::order_form::OrderForm;
use crate::{actors::book_service::BookData, models::book::Book, pages::OrderBook};
use askama::Template;

#[derive(Template, Debug, Clone)]
#[template(path = "book.html")]
pub struct BookHtml {
    pub title: String,
    pub book_data: OrderBook,
    pub order_form: OrderForm,
}

impl BookHtml {
    pub fn new(book: Book, book_data: &BookData) -> Self {
        Self {
            title: book.title,
            book_data: OrderBook::from(book_data),
            order_form: OrderForm::new(book_data.book_id),
        }
    }
}
