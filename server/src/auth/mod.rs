//! Authentication extractor and routes.

pub mod handlers;
pub mod model;
pub mod password;

use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    routing::{get, post},
    Router,
};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

/// An authenticated user, yielded only when a valid session exists. Handlers
/// that require auth take `AuthUser`, never `Option<AuthUser>`.
pub struct AuthUser(pub Uuid);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let user_id = session
            .get::<Uuid>("user_id")
            .await
            .map_err(AppError::from)?;
        user_id.map(AuthUser).ok_or(AppError::Unauthorized)
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/me", get(handlers::me))
}
