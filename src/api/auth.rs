use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum::response::Response;
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;

use crate::app_state::AppState;
use crate::models;
use crate::models::user::User;

use super::api_error::ApiError;

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
            return Err(ApiError::Authentication.into_response());
        };
        match models::user::User::check_login(&state.pool, x.username(), x.password()).await {
            Some(user) => Ok(Self(user)),
            None => Err(ApiError::Authentication.into_response()),
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
        let user =
            models::user::User::check_login(&state.pool, header.username(), header.password())
                .await;
        Ok(Self(user))
    }
}
