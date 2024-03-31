use axum::response::{Html, IntoResponse};

use super::templates::home;
use crate::auth::SessionExtractor;

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(user) => Html(home::build(&user.username)).into_response(),
        None => Html(home::build("")).into_response(),
    }
}
