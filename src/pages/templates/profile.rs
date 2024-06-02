use askama::Template;
use lobster::Balance;

use super::{open_orders, positions};

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Profile {
    username: String,
    balance: String,
    positions: positions::Positions,
    open_orders: open_orders::OpenOrders,
}

impl Profile {
    pub fn new(
        username: String,
        balance: Balance,
        positions: positions::Positions,
        open_orders: open_orders::OpenOrders,
    ) -> Self {
        Self {
            username,
            balance: format!("${:.2}", balance as f32 / 10000.0),
            positions,
            open_orders,
        }
    }
}
