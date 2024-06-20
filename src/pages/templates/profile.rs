use askama::Template;

use crate::models::user::User;

use super::{open_orders, positions};

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Profile {
    username: String,
    balance: String,
    available: String,
    positions: positions::Positions,
    open_orders: open_orders::OpenOrders,
}

impl Profile {
    pub fn new(
        user: User,
        positions: positions::Positions,
        open_orders: open_orders::OpenOrders,
    ) -> Self {
        Self {
            username: user.username,
            balance: format!("${:.2}", user.balance as f32 / 10000.0),
            available: format!("${:.2}", user.available as f32 / 10000.0),
            positions,
            open_orders,
        }
    }
}
