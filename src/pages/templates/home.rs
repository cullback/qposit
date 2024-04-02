use askama::Template;

use crate::models::{book::Book, market::Market};

#[derive(Template)]
#[template(path = "home.html")]
pub struct Component<'a> {
    username: &'a str,
    markets: Vec<(Market, Vec<Book>)>,
}

pub fn build(username: &str, markets: Vec<(Market, Vec<Book>)>) -> String {
    Component { username, markets }.render().unwrap()
}
