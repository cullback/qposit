use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use axum_extra::{
    extract::{cookie::Cookie, CookieJar},
    headers::UserAgent,
    TypedHeader,
};
use serde::Deserialize;
use tracing::info;

use crate::{
    app_state::{current_time_micros, AppState},
    auth::{self, SessionExtractor},
    models::session::Session,
};

use super::templates::{login_form, login_page};

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(_) => Redirect::to("/").into_response(),
        None => login_page::LoginPage::new(String::new(), String::new()).into_response(),
    }
}

#[derive(Deserialize, Debug)]
pub struct Credentials {
    username: String,
    password: String,
}

pub async fn post(
    jar: CookieJar,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(form): Form<Credentials>,
) -> impl IntoResponse {
    let timestamp = current_time_micros();

    match auth::login(
        &state.db,
        &form.username,
        &form.password,
        timestamp,
        addr.ip().to_string(),
        user_agent,
    )
    .await
    {
        Some(cookie) => {
            info!("User {} logged in", form.username);
            ([("HX-Redirect", "/")], jar.add(cookie)).into_response()
        }
        None => login_form::LoginForm::new("Incorrect username / password combination".to_string())
            .into_response(),
    }
}

pub async fn delete(jar: CookieJar, State(state): State<AppState>) -> impl IntoResponse {
    info!("DELETE /login");
    if let Some(cookie) = jar.get("session_id") {
        Session::delete_by_id(&state.db, cookie.value())
            .await
            .expect("failed to delete session id from database");
    }
    (
        [("HX-Redirect", "/")],
        jar.remove(Cookie::build("session_id")),
    )
        .into_response()
}

pub async fn delete_by_id(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    Session::delete_by_id(&state.db, &session_id).await.unwrap();
    Html("").into_response()
}
