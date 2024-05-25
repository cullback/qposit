use super::templates::about_page::AboutPage;
use crate::authentication::SessionExtractor;
use axum::response::IntoResponse;

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(user) => AboutPage::new(user.username),
        None => AboutPage::default(),
    }
}
