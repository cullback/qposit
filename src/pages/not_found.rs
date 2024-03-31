use axum::response::{Html, IntoResponse};

use super::templates::not_found;
use crate::auth::SessionExtractor;

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(user) => Html(not_found::build(&user.username)).into_response(),
        None => Html(not_found::build("")).into_response(),
    }
}
