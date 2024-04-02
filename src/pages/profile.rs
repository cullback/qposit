use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use sqlx::SqlitePool;

use crate::{auth::SessionExtractor, models::session::Session};

use super::templates::profile;

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    db: Extension<SqlitePool>,
) -> impl IntoResponse {
    match user {
        Some(user) => {
            let Ok(sessions) = Session::get_all_for_user(&db, user.id).await else {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            };

            Html(profile::build(&user.username, &sessions)).into_response()
        }
        None => Redirect::to("/").into_response(),
    }
}
