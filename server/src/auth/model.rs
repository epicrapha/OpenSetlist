//! Auth request/response and persistence models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::workspace::roles::Role;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// A membership as exposed to the current user. `workspace_id` identifies the
/// tenant; this is all `/auth/me` needs.
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct MembershipView {
    pub workspace_id: Uuid,
    pub role: Role,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user: UserResponse,
    pub memberships: Vec<MembershipView>,
}
