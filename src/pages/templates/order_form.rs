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
        OrderForm {
            book_id,
            quantity: "".to_string(),
            price: "".to_string(),
            quantity_message: "".to_string(),
            price_message: "".to_string(),
            message: "".to_string(),
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
        OrderForm {
            book_id,
            quantity,
            price,
            quantity_message,
            price_message,
            message,
        }
    }
}
