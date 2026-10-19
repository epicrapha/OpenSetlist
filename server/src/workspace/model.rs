//! Workspace, membership and invite models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::roles::Role;

#[derive(Debug, sqlx::FromRow)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub settings: Value,
    pub plan: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceResponse {
    pub id: Uuid,
    pub name: String,
    pub settings: Value,
    pub plan: String,
    pub created_at: DateTime<Utc>,
}

impl From<Workspace> for WorkspaceResponse {
    fn from(workspace: Workspace) -> Self {
        Self {
            id: workspace.id,
            name: workspace.name,
            settings: workspace.settings,
            plan: workspace.plan,
            created_at: workspace.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Membership {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub role: Role,
}

/// An invite. The JSON shape is the frozen contract in PLAN.md §3.2, so `Invite`
/// is used directly as the response (no separate response type to drift).
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Invite {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub role: Role,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InviteListResponse {
    pub invites: Vec<Invite>,
}

#[derive(Debug, Serialize)]
pub struct AcceptInviteResponse {
    pub workspace_id: Uuid,
    pub role: Role,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: String,
}
