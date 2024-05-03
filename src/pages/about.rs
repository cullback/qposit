use super::templates::about::AboutPage;
use crate::auth::SessionExtractor;
use axum::response::IntoResponse;

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(user) => AboutPage {
            username: user.username,
        },
        None => AboutPage {
            username: "".to_string(),
        },
    }
}
