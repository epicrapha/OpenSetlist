//! Workspace routes.

pub mod handlers;
pub mod model;
pub mod roles;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/workspaces", post(handlers::create))
        .route("/workspaces/{id}", get(handlers::get))
        .route(
            "/workspaces/{id}/invites",
            get(handlers::list_invites).post(handlers::create_invite),
        )
        .route("/invites/{token}/accept", post(handlers::accept_invite))
}
