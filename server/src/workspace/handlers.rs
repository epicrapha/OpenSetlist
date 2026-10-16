//! Workspace HTTP handlers: create, get, create invite.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

use super::model::{
    CreateInviteRequest, CreateWorkspaceRequest, Invite, InviteResponse, Membership, Workspace,
    WorkspaceResponse,
};
use super::roles::Role;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::state::AppState;

const MEMBERSHIP_BY_USER: &str = "select id, user_id, workspace_id, role from memberships \
     where user_id = $1 and workspace_id = $2";

async fn membership_for(
    tx: &mut crate::tenant::TenantTx<'_>,
    user_id: Uuid,
    workspace_id: Uuid,
) -> Result<Option<Membership>, AppError> {
    let membership = sqlx::query_as::<_, Membership>(MEMBERSHIP_BY_USER)
        .bind(user_id)
        .bind(workspace_id)
        .fetch_optional(&mut **tx)
        .await?;
    Ok(membership)
}

/// POST /api/v1/workspaces — create a workspace and the owner membership
/// atomically, with the RLS context set to the new workspace id.
pub async fn create(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<(StatusCode, Json<WorkspaceResponse>), AppError> {
    let AuthUser(user_id) = user;
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }

    let workspace_id = Uuid::new_v4();
    let mut tx = state.scoped(Some(workspace_id), Some(user_id)).await?;

    let workspace = sqlx::query_as::<_, Workspace>(
        "insert into workspaces (id, name, plan) values ($1, $2, 'free') \
         returning id, name, settings, plan, created_at",
    )
    .bind(workspace_id)
    .bind(req.name.trim())
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("insert into memberships (user_id, workspace_id, role) values ($1, $2, 'owner')")
        .bind(user_id)
        .bind(workspace_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(workspace.into())))
}

/// GET /api/v1/workspaces/:id — members only. Non-members get 404 so the
/// endpoint does not reveal whether a workspace exists.
pub async fn get(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let AuthUser(user_id) = user;
    let mut tx = state.scoped(Some(id), Some(user_id)).await?;

    if membership_for(&mut tx, user_id, id).await?.is_none() {
        return Err(AppError::NotFound);
    }

    let workspace = sqlx::query_as::<_, Workspace>(
        "select id, name, settings, plan, created_at from workspaces where id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;

    workspace
        .map(|workspace| Json(workspace.into()))
        .ok_or(AppError::NotFound)
}

/// POST /api/v1/workspaces/:id/invites — owner/admin only.
pub async fn create_invite(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<InviteResponse>), AppError> {
    let AuthUser(user_id) = user;
    let role = Role::parse(&req.role).ok_or_else(|| AppError::Validation("invalid role".into()))?;
    let email = req.email.trim();
    if email.len() < 3 || !email.contains('@') {
        return Err(AppError::Validation("a valid email is required".into()));
    }

    let mut tx = state.scoped(Some(id), Some(user_id)).await?;

    // Non-members see 404, members without owner/admin see 403.
    let membership = membership_for(&mut tx, user_id, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if !membership.role.at_least(Role::Admin) {
        return Err(AppError::Forbidden);
    }

    let token = Uuid::new_v4().simple().to_string();
    let expires_at = Utc::now() + Duration::days(7);

    let invite = sqlx::query_as::<_, Invite>(
        "insert into invites (workspace_id, email, role, token, expires_at) \
         values ($1, $2::citext, $3::membership_role, $4, $5) \
         returning id, workspace_id, email::text as email, role, token, expires_at, accepted_at",
    )
    .bind(id)
    .bind(email)
    .bind(role.as_str())
    .bind(&token)
    .bind(expires_at)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(invite.into())))
}
