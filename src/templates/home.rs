use askama::Template;

use crate::models::market::Market;

#[derive(Template)]
#[template(path = "home.html")]
pub struct Component<'a> {
    markets: &'a [Market],
}

pub fn build(name: &str, markets: &[Market]) -> String {
    Component { markets }.render().unwrap()
}

