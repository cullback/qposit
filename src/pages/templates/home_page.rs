use askama::Template;

use crate::models::{book::Book, market::Market};

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomePage {
    pub username: String,
    pub markets: Vec<(Market, Vec<Book>)>,
}
