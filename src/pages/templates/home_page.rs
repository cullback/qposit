use askama::Template;

use crate::models::{book::Book, market::Market};

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomePage {
    username: String,
    markets: Vec<(Market, Vec<Book>)>,
}

impl HomePage {
    pub fn new(username: String, markets: Vec<(Market, Vec<Book>)>) -> Self {
        Self { username, markets }
    }
}
