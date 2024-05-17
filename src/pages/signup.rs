use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::extract::{ConnectInfo, State};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::CookieJar;
use axum_extra::headers::UserAgent;
use axum_extra::TypedHeader;
use rand::rngs::OsRng;
use serde::Deserialize;
use std::net::SocketAddr;
use tracing::warn;

use super::templates::{signup, signup_form};
use crate::app_state::{current_time_micros, AppState};
use crate::auth::{self, SessionExtractor};
use crate::models;

pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(_) => Redirect::to("/").into_response(),
        None => signup::Component::new().into_response(),
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

    let username_message = validate_username(&form.username);
    let password_message = validate_password(&form.password);
    if !username_message.is_empty() || !password_message.is_empty() {
        return signup_form::SignupForm::new(form.username, username_message, password_message, String::new())
            .into_response();
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let Ok(password_hash) = argon2.hash_password(form.password.as_bytes(), &salt) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let user = models::user::User {
        id: 0,
        username: form.username.clone(),
        password_hash: password_hash.to_string(),
        created_at: timestamp,
        balance: 0,
    };

    match user.insert(&state.db).await {
        Ok(user_id) => {
            let cookie =
                auth::create_session(&state.db, user_id, timestamp, addr.to_string(), user_agent)
                    .await;
            ([("HX-Redirect", "/")], jar.add(cookie)).into_response()
        }
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            signup_form::SignupForm::new(
                form.username,
                "Username already taken".to_string(),
                String::new(),
                String::new(),
            )
            .into_response()
        }
        Err(err) => {
            warn!("internal server error {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn validate_username(username: &str) -> String {
    if username.len() < 5 || username.len() > 20 || !username.chars().all(char::is_alphanumeric) {
        "Username must be between 5 and 20 characters, and only contain letters / numbers."
    } else {
        ""
    }
    .to_string()
}

fn validate_password(password: &str) -> String {
    if password.len() < 8 || password.len() > 60 || !password.is_ascii() {
        "Password must be between 8 and 60 characters and only contain ascii characters."
    } else {
        ""
    }
    .to_string()
}
