use super::templates::about_page::AboutPage;
use crate::auth::SessionExtractor;
use axum::response::IntoResponse;

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(user) => AboutPage {
            username: user.username,
        },
        None => AboutPage {
            username: String::new(),
        },
    }
}
