use askama::Template;
use lobster::EventId;

#[derive(Template, Debug, Clone)]
#[template(path = "order_form.html")]
pub struct OrderForm {
    event_id: EventId,
    quantity: String,
    price: String,
    quantity_message: String,
    price_message: String,
    message: String,
}

impl OrderForm {
    pub fn new(event_id: EventId) -> Self {
        Self {
            event_id,
            quantity: String::new(),
            price: String::new(),
            quantity_message: String::new(),
            price_message: String::new(),
            message: String::new(),
        }
    }
    pub const fn with_messages(
        event_id: EventId,
        quantity: String,
        price: String,
        quantity_message: String,
        price_message: String,
        message: String,
    ) -> Self {
        Self {
            event_id,
            quantity,
            price,
            quantity_message,
            price_message,
            message,
        }
    }
}
