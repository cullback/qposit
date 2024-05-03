use askama::Template;
use exchange::BookId;

#[derive(Template)]
#[template(path = "order_form.html")]
pub struct OrderForm {
    book_id: BookId,
    quantity: String,
    price: String,
    quantity_message: String,
    price_message: String,
    message: String,
}

impl OrderForm {
    pub fn new(book_id: BookId) -> Self {
        Self {
            book_id,
            quantity: String::new(),
            price: String::new(),
            quantity_message: String::new(),
            price_message: String::new(),
            message: String::new(),
        }
    }
    pub fn with_messages(
        book_id: BookId,
        quantity: String,
        price: String,
        quantity_message: String,
        price_message: String,
        message: String,
    ) -> Self {
        Self {
            book_id,
            quantity,
            price,
            quantity_message,
            price_message,
            message,
        }
    }
}
