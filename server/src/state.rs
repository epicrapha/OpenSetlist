//! Shared application state, passed to handlers via `State<Arc<AppState>>`.

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    /// A valid Argon2id hash used to equalise login timing for unknown users,
    /// so the login response cannot be used to enumerate accounts.
    pub dummy_password_hash: String,
}
