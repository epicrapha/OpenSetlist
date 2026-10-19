//! Setlist routes (PLAN.md §3.3).
//!
//! Workspace-scoped routes resolve the tenant from `X-Workspace-Id`; the public
//! share route is deliberately unauthenticated and header-free.

pub mod handlers;
pub mod model;

use std::sync::Arc;

use axum::{
    routing::{get, patch, post, put},
    Router,
};

use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/setlists", get(handlers::list).post(handlers::create))
        .route(
            "/setlists/{id}",
            get(handlers::get)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
        .route("/setlists/{id}/duplicate", post(handlers::duplicate))
        .route("/setlists/{id}/items", post(handlers::create_item))
        .route("/setlists/{id}/items/order", put(handlers::reorder))
        .route(
            "/setlists/{id}/items/{item_id}",
            patch(handlers::update_item).delete(handlers::delete_item),
        )
        .route(
            "/setlists/{id}/share",
            post(handlers::create_share).delete(handlers::revoke_share),
        )
        .route("/shared/setlists/{token}", get(handlers::shared))
}
