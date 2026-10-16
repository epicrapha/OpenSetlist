//! Song library routes, matching the frozen contract in PLAN.md §3.1.
//!
//! All routes are workspace-scoped via the `X-Workspace-Id` header. Reads need
//! any membership (viewer+); writes call `WorkspaceScope::require_leader` and
//! return 403 for members/viewers.

pub mod arrangements;
pub mod assets;
pub mod handlers;
pub mod model;
pub mod scope;

use std::sync::Arc;

use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, patch, post},
    Router,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState, tenant::TenantTx};

/// Confirm a song exists inside the current tenant, else 404. Used before
/// attaching child rows so an id from another workspace cannot be referenced.
pub(crate) async fn ensure_song(tx: &mut TenantTx<'_>, song_id: Uuid) -> Result<(), AppError> {
    let exists: Option<Uuid> = sqlx::query_scalar("select id from songs where id = $1")
        .bind(song_id)
        .fetch_optional(&mut **tx)
        .await?;
    exists.map(|_| ()).ok_or(AppError::NotFound)
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/songs", get(handlers::list).post(handlers::create))
        .route(
            "/songs/{id}",
            get(handlers::get)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
        .route("/songs/{id}/duplicate", post(handlers::duplicate))
        .route("/songs/{id}/export", get(handlers::export))
        .route(
            "/songs/{id}/arrangements",
            get(arrangements::list).post(arrangements::create),
        )
        .route(
            "/arrangements/{id}",
            patch(arrangements::update).delete(arrangements::delete),
        )
        .route(
            "/songs/{id}/assets",
            get(assets::list).post(assets::create_link),
        )
        .route(
            "/songs/{id}/assets/pdf",
            post(assets::upload_pdf)
                // Allow multipart overhead above the 10 MB file cap so the
                // handler (not the body limit) produces the 413 for oversized
                // files, with our JSON error shape.
                .layer(DefaultBodyLimit::max(
                    assets::MAX_UPLOAD_BYTES + 1024 * 1024,
                )),
        )
        .route("/assets/{id}", delete(assets::delete))
        .route("/assets/{id}/file", get(assets::download))
}
