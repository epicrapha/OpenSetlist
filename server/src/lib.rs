//! OpenSetlist server library: router construction and shared setup.

pub mod auth;
pub mod config;
pub mod error;
pub mod setlist;
pub mod song;
pub mod state;
pub mod tenant;
pub mod token;
pub mod workspace;

use std::sync::Arc;

use axum::{routing::get, Json, Router};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tower_sessions::{cookie::SameSite, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

use crate::{config::Config, state::AppState};

/// Connect to Postgres, run SQL migrations and the session-store migration,
/// and build the shared state.
pub async fn build_state(config: Config) -> anyhow::Result<Arc<AppState>> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;

    // One valid hash reused for unknown-user logins so failed logins cost the
    // same as real ones (anti-enumeration).
    let dummy_password_hash =
        auth::password::hash("opensetlist-dummy-password".to_string()).await?;

    Ok(Arc::new(AppState {
        pool,
        config: Arc::new(config),
        dummy_password_hash,
    }))
}

/// Build the HTTP router, applying the session layer.
pub fn build_router(state: Arc<AppState>) -> Router {
    let session_store = PostgresStore::new(state.pool.clone());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("opensetlist.sid")
        .with_same_site(SameSite::Lax)
        .with_secure(state.config.cookie_secure);

    let api = Router::new()
        .merge(auth::routes())
        .merge(workspace::routes())
        .merge(song::routes())
        .merge(setlist::routes());

    Router::new()
        .route("/health", get(health))
        .nest("/api/v1", api)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
