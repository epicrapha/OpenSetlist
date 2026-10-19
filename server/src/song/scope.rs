//! Workspace scope extractor.
//!
//! The frozen M1 paths (`/api/v1/songs`, …) carry no workspace id, so the
//! tenant is resolved from the `X-Workspace-Id` header. The extractor requires
//! auth, parses the header, and looks up the caller's membership. Non-members
//! get 404 — the endpoint never confirms whether a workspace exists.

use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{header::HeaderName, request::Parts},
};
use uuid::Uuid;

use crate::{auth::AuthUser, error::AppError, state::AppState, workspace::roles::Role};

pub const WORKSPACE_HEADER: HeaderName = HeaderName::from_static("x-workspace-id");

/// An authenticated caller proven to be a member of a specific workspace.
pub struct WorkspaceScope {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
}

impl WorkspaceScope {
    /// Reads require any membership (viewer+); the role is already viewer or
    /// above by construction.
    pub fn require_leader(&self) -> Result<(), AppError> {
        if self.role.at_least(Role::Leader) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

impl FromRequestParts<Arc<AppState>> for WorkspaceScope {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user_id) = AuthUser::from_request_parts(parts, state).await?;

        let raw = parts
            .headers
            .get(&WORKSPACE_HEADER)
            .ok_or_else(|| AppError::Validation("X-Workspace-Id header is required".into()))?
            .to_str()
            .map_err(|_| AppError::Validation("X-Workspace-Id is not valid UTF-8".into()))?;
        let workspace_id = Uuid::parse_str(raw)
            .map_err(|_| AppError::Validation("X-Workspace-Id must be a UUID".into()))?;

        let mut tx = state.scoped(Some(workspace_id), Some(user_id)).await?;
        let role: Option<Role> = sqlx::query_scalar(
            "select role from memberships where user_id = $1 and workspace_id = $2",
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_optional(&mut *tx)
        .await?;

        // Non-members (and non-existent workspaces) are indistinguishable: 404.
        let role = role.ok_or(AppError::NotFound)?;
        Ok(WorkspaceScope {
            workspace_id,
            user_id,
            role,
        })
    }
}
