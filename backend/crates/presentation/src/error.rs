//! Presentation-layer error adapter: maps `ApplicationError` to HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use ffm_application::error::ApplicationError;

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

pub struct ApiError(pub ApplicationError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self.0 {
            ApplicationError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            ApplicationError::Conflict(m) => (StatusCode::CONFLICT, m),
            ApplicationError::Parse(m) => (StatusCode::BAD_REQUEST, m),
            ApplicationError::InvalidState(m) => (StatusCode::BAD_REQUEST, m),
            ApplicationError::Config(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
            ApplicationError::Database(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
            ApplicationError::Io(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
            ApplicationError::FireflyClient(m) => (StatusCode::BAD_GATEWAY, m),
        };
        let body = ErrorBody { error: msg };
        (status, Json(body)).into_response()
    }
}
