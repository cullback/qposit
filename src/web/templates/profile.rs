use askama::Template;

use crate::models::user::User;

use super::{format_balance_to_dollars, format_timestamp_as_string, open_orders, positions};

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Profile {
    username: String,
    profile_username: String,
    created_at: String,
    balance: String,
    available: String,
    positions: positions::Positions,
    open_orders: open_orders::OpenOrders,
}

impl Profile {
    pub fn new(
        logged_in_user: Option<User>,
        user: User,
        positions: positions::Positions,
        open_orders: open_orders::OpenOrders,
    ) -> Self {
        Self {
            username: logged_in_user.map(|u| u.username).unwrap_or_default(),
            profile_username: user.username,
            created_at: format_timestamp_as_string(user.created_at),
            balance: format_balance_to_dollars(user.balance),
            available: format_balance_to_dollars(user.available),
            positions,
            open_orders,
        }
    }
}
