use argon2::PasswordVerifier;
use argon2::{Argon2, PasswordHash};
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum_extra::extract::{cookie::Cookie, cookie::SameSite, CookieJar};
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::{Authorization, UserAgent};
use axum_extra::TypedHeader;
use sqlx::{Executor, Sqlite, SqlitePool};
use tracing::{error, warn};

use crate::app_state::AppState;
use crate::models::session::Session;
use crate::models::user::User;
use lobster::Timestamp;
use lobster::UserId;

/// Generates a random 128-bit hex string.
fn generate_session_id() -> String {
    format!("{:#018x}", rand::random::<u128>())
}

fn build_session_cookie(session_id: &str) -> Cookie<'static> {
    const IS_RELEASE: bool = !cfg!(debug_assertions);
    Cookie::build(("session_id", session_id))
        .path("/")
        .same_site(SameSite::Strict)
        .secure(IS_RELEASE)
        .http_only(true)
        .max_age(time::Duration::WEEK)
        .build()
        .into_owned()
}

/// Create a session for the user and return a cookie for it.
pub async fn create_session<'c, E: Executor<'c, Database = Sqlite>>(
    pool: E,
    user_id: UserId,
    time: Timestamp,
    ip_address: String,
    user_agent: UserAgent,
) -> Cookie<'static> {
    let id = generate_session_id();
    let session = Session {
        id: id.clone(),
        user_id,
        ip_address,
        user_agent: user_agent.to_string(),
        created_at: time,
        expires_at: 0, // todo
    };
    session.insert(pool).await.unwrap();
    build_session_cookie(&id)
}

/// Checks a usernames+password combination using the database and returns the user if it is valid.
/// Returns `None` if the user does not exist or the password is incorrect.
async fn check_login(pool: &SqlitePool, username: &str, password: &str) -> Option<User> {
    match User::get_by_username(pool, username).await {
        Ok(user) => {
            let parsed_hash = PasswordHash::new(&user.password_hash).expect("Failed to parsh hash");
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .ok()
                .map(|()| user)
        }
        Err(sqlx::Error::RowNotFound) => return None,
        Err(err) => {
            error!(err = ?err, "Failed to get user");
            return None;
        }
    }
}

/// Authenticate user and create a new session id.
pub async fn login(
    db: &SqlitePool,
    username: &str,
    password: &str,
    time: Timestamp,
    ip_address: String,
    user_agent: UserAgent,
) -> Option<Cookie<'static>> {
    let user = check_login(db, username, password).await?;

    let cookie = create_session(db, user.id, time, ip_address, user_agent).await;
    Some(cookie)
}

/// None if the session is invalid.
pub struct SessionExtractor(pub Option<User>);

#[async_trait]
impl FromRequestParts<AppState> for SessionExtractor {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        let Some(session_id) = jar.get("session_id").map(Cookie::value) else {
            return Ok(Self(None));
        };
        let session = match Session::get_by_id(&state.pool, session_id).await {
            Ok(Some(session)) => session,
            Ok(None) => return Ok(Self(None)),
            Err(err) => {
                warn!(err = ?err, "Failed to get session");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };
        let user = match User::get_by_id(&state.pool, session.user_id).await {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => return Ok(Self(None)),
            Err(err) => {
                warn!(err = ?err, "Failed to get user");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };

        Ok(Self(Some(user)))
    }
}

pub struct BasicAuthExtractor(pub User);

#[async_trait]
impl FromRequestParts<AppState> for BasicAuthExtractor {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Ok(TypedHeader(Authorization(x))) =
            TypedHeader::<Authorization<Basic>>::from_request_parts(parts, state).await
        else {
            return Err(StatusCode::UNAUTHORIZED.into_response());
        };
        match check_login(&state.pool, x.username(), x.password()).await {
            Some(user) => Ok(Self(user)),
            None => Err(StatusCode::UNAUTHORIZED.into_response()),
        }
    }
}

pub struct OptionalBasicAuth(pub Option<User>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalBasicAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Ok(TypedHeader(Authorization(header))) =
            TypedHeader::<Authorization<Basic>>::from_request_parts(parts, state).await
        else {
            return Ok(Self(None));
        };
        let user = check_login(&state.pool, header.username(), header.password()).await;
        Ok(Self(user))
    }
}
