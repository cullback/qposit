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

use super::templates::signup;
use crate::actors::matcher_request::MatcherRequest;
use crate::app_state::{current_time_micros, AppState};
use crate::authentication::{self, SessionExtractor};
use crate::models;
use crate::models::invite::Invite;
use crate::models::user::User;

/// Get the signup page, or redirect to the home page if the user is already logged in.
pub async fn get(SessionExtractor(user): SessionExtractor) -> impl IntoResponse {
    match user {
        Some(_) => Redirect::to("/").into_response(),
        None => signup::Component::default().into_response(),
    }
}

#[derive(Deserialize, Debug)]
pub struct FormPayload {
    username: String,
    password: String,
    invite_code: String,
}

/// Handle a signup request.
pub async fn post(
    jar: CookieJar,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(form): Form<FormPayload>,
) -> impl IntoResponse {
    let timestamp = current_time_micros();

    if let Err(page) = validate_inputs(&form) {
        return page.into_response();
    }

    let password_hash = generate_password_hash(&form.password);
    let user = models::user::User {
        id: 0,
        username: form.username.clone(),
        password_hash,
        created_at: timestamp,
        balance: 0,
    };

    let mut tx = state.pool.begin().await.unwrap();
    let user_id = match user.insert(&mut *tx).await {
        Ok(user_id) => user_id,
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            return signup::SignupForm::new(
                form.username,
                "Username already taken".to_string(),
                String::new(),
                String::new(),
            )
            .into_response();
        }
        Err(err) => {
            warn!("internal server error {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match Invite::check_and_claim(&mut *tx, &form.invite_code, user_id).await {
        Ok(Some(_)) => {
            tx.commit().await.unwrap();
            let cookie = authentication::create_session(
                &state.pool,
                user_id,
                timestamp,
                addr.to_string(),
                user_agent,
            )
            .await;

            let initial_amount = 10000 * 500; // TODO
            let rows_affected = User::deposit(&state.pool, user_id, initial_amount)
                .await
                .unwrap();
            assert_eq!(rows_affected, 1);
            let req = MatcherRequest::deposit(user_id, initial_amount);
            state.cmd_send.send(req).await.unwrap();
            ([("HX-Redirect", "/")], jar.add(cookie)).into_response()
        }
        Ok(None) => signup::SignupForm::new(
            form.username,
            String::new(),
            String::new(),
            "Invite code does not exist or has already been claimed".to_string(),
        )
        .into_response(),
        Err(err) => {
            warn!("internal server error {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn generate_password_hash(plaintext_password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(plaintext_password.as_bytes(), &salt)
        .unwrap();
    password_hash.to_string()
}

fn validate_inputs(form: &FormPayload) -> Result<(), signup::SignupForm> {
    let username_message = validate_username(&form.username);
    let password_message = validate_password(&form.password);
    if !username_message.is_empty() || !password_message.is_empty() {
        Err(signup::SignupForm::new(
            form.username.clone(),
            username_message,
            password_message,
            String::new(),
        ))
    } else {
        Ok(())
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
