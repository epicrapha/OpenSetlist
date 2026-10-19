//! Central application error type.
//!
//! Every handler returns `Result<_, AppError>` and every failure serialises to
//! the frozen contract shape: `{"error": {"code": "...", "message": "..."}}`.
//! Internal errors are logged with their source but never leaked to clients.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    PayloadTooLarge(String),
    #[error("{0}")]
    UnsupportedMedia(String),
    #[error(transparent)]
    Session(#[from] tower_sessions::session::Error),
    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Wrap any displayable value as an internal error (logged, not exposed).
    pub fn internal(message: impl std::fmt::Display) -> Self {
        AppError::Internal(anyhow::anyhow!("{message}"))
    }

    fn code(&self) -> &'static str {
        match self {
            AppError::Unauthorized => "unauthorized",
            AppError::Forbidden => "forbidden",
            AppError::NotFound => "not_found",
            AppError::Validation(_) => "validation_error",
            AppError::BadRequest(_) => "bad_request",
            AppError::Conflict(_) => "conflict",
            AppError::PayloadTooLarge(_) => "payload_too_large",
            AppError::UnsupportedMedia(_) => "unsupported_media_type",
            _ => "internal_error",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::UnsupportedMedia(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let code = self.code();
        let message = match &self {
            AppError::Validation(m) | AppError::Conflict(m) => m.clone(),
            AppError::BadRequest(m) => m.clone(),
            AppError::PayloadTooLarge(m) | AppError::UnsupportedMedia(m) => m.clone(),
            AppError::Unauthorized => "authentication required".to_string(),
            AppError::Forbidden => "not permitted".to_string(),
            AppError::NotFound => "not found".to_string(),
            other => {
                tracing::error!(error = %other, "request failed");
                "internal server error".to_string()
            }
        };
        (
            status,
            Json(json!({ "error": { "code": code, "message": message } })),
        )
            .into_response()
    }
}
