use axum::{
    extract::{rejection::JsonRejection, FromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

impl<T> IntoResponse for ApiJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

// The kinds of errors we can hit in our application.
pub enum ApiError {
    // The request body contained invalid JSON
    JsonRejection(JsonRejection),
    MatcherRequest(lobster::RejectReason),
    Authentication,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
        }

        let (status, message) = match self {
            ApiError::JsonRejection(rejection) => {
                // This error is caused by bad user input so don't log it
                (rejection.status(), rejection.body_text())
            }
            ApiError::MatcherRequest(reason) => (StatusCode::OK, format!("{reason:?}")),
            ApiError::Authentication => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        };
        (status, ApiJson(ErrorResponse { error: message })).into_response()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}
