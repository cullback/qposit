use axum::response::{Html, IntoResponse};

use crate::auth::SessionExtractor;

use askama::Template;

#[derive(Template)]
#[template(path = "not_found.html")]
pub struct Component<'a> {
    username: &'a str,
}

pub fn build(username: &str) -> String {
    Component { username }.render().unwrap()
}

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(user) => Html(build(&user.username)).into_response(),
        None => Html(build("")).into_response(),
    }
}
