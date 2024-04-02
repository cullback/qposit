use axum::response::{Html, IntoResponse};

use crate::auth::SessionExtractor;

use super::templates::about;

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(user) => Html(about::build(&user.username)),
        None => Html(about::build("")),
    }
}
