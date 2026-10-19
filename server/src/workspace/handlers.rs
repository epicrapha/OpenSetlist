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
    AcceptInviteResponse, CreateInviteRequest, CreateWorkspaceRequest, Invite, InviteListResponse,
    Membership, Workspace, WorkspaceResponse,
};
use super::roles::Role;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::state::AppState;

const MEMBERSHIP_BY_USER: &str = "select id, user_id, workspace_id, role from memberships \
     where user_id = $1 and workspace_id = $2";

const INVITE_COLUMNS: &str = "id, workspace_id, email::text as email, role, token, \
     expires_at, accepted_at, created_at";

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

/// GET /api/v1/workspaces/:id/invites — owner/admin only. Non-members get 404
/// (like the sibling workspace endpoints, so the response does not reveal that
/// the workspace exists); members without owner/admin get 403.
pub async fn list_invites(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<InviteListResponse>, AppError> {
    let AuthUser(user_id) = user;
    let mut tx = state.scoped(Some(id), Some(user_id)).await?;

    let membership = membership_for(&mut tx, user_id, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if !membership.role.at_least(Role::Admin) {
        return Err(AppError::Forbidden);
    }

    // Pending invites first, then most recent.
    let sql = format!(
        "select {INVITE_COLUMNS} from invites where workspace_id = $1 \
         order by (accepted_at is not null) asc, created_at desc"
    );
    let invites = sqlx::query_as::<_, Invite>(&sql)
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

    Ok(Json(InviteListResponse { invites }))
}

/// POST /api/v1/workspaces/:id/invites — owner/admin only. `role` may be any
/// role except `owner` (an invite must never mint a second owner).
pub async fn create_invite(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<Invite>), AppError> {
    let AuthUser(user_id) = user;
    let role = Role::parse(&req.role).ok_or_else(|| AppError::Validation("invalid role".into()))?;
    if role == Role::Owner {
        return Err(AppError::Validation(
            "role must be one of admin, leader, member, viewer".into(),
        ));
    }
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

    let token = crate::token::random_token();
    let expires_at = Utc::now() + Duration::days(7);

    let sql = format!(
        "insert into invites (workspace_id, email, role, token, expires_at) \
         values ($1, $2::citext, $3::membership_role, $4, $5) returning {INVITE_COLUMNS}"
    );
    let invite = sqlx::query_as::<_, Invite>(&sql)
        .bind(id)
        .bind(email)
        .bind(role.as_str())
        .bind(&token)
        .bind(expires_at)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(invite)))
}

/// POST /api/v1/invites/:token/accept — any authenticated user whose email
/// matches the invite. The token is resolved before the workspace is known, so
/// the transaction starts under the `invites_by_token` RLS policy, then scopes
/// itself to the invite's workspace for the acceptance writes.
pub async fn accept_invite(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(token): Path<String>,
) -> Result<(StatusCode, Json<AcceptInviteResponse>), AppError> {
    let AuthUser(user_id) = user;
    if token.trim().is_empty() {
        return Err(AppError::NotFound);
    }

    let mut tx = state.scoped_invite_token(&token).await?;
    let sql = format!("select {INVITE_COLUMNS} from invites where token = $1");
    let invite = sqlx::query_as::<_, Invite>(&sql)
        .bind(&token)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;

    // Scope the rest of this transaction to the invite's workspace.
    sqlx::query("select set_config('app.current_tenant_id', $1, true)")
        .bind(invite.workspace_id.to_string())
        .execute(&mut *tx)
        .await?;

    if invite.accepted_at.is_some() {
        return Err(AppError::Conflict(
            "invite has already been accepted".into(),
        ));
    }
    if invite.expires_at < Utc::now() {
        return Err(AppError::Conflict("invite has expired".into()));
    }

    let already_member: Option<Uuid> =
        sqlx::query_scalar("select id from memberships where user_id = $1 and workspace_id = $2")
            .bind(user_id)
            .bind(invite.workspace_id)
            .fetch_optional(&mut *tx)
            .await?;
    if already_member.is_some() {
        return Err(AppError::Conflict(
            "you are already a member of this workspace".into(),
        ));
    }

    // Email-bound: the caller's account email must match the invite.
    let caller_email: Option<String> =
        sqlx::query_scalar("select email::text from users where id = $1")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;
    let caller_email = caller_email.ok_or(AppError::Unauthorized)?;
    if !caller_email
        .trim()
        .eq_ignore_ascii_case(invite.email.trim())
    {
        return Err(AppError::Forbidden);
    }

    // Accept + join are one atomic write under RLS.
    sqlx::query("update invites set accepted_at = now() where id = $1")
        .bind(invite.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "insert into memberships (user_id, workspace_id, role) \
         values ($1, $2, $3::membership_role)",
    )
    .bind(user_id)
    .bind(invite.workspace_id)
    .bind(invite.role.as_str())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(AcceptInviteResponse {
            workspace_id: invite.workspace_id,
            role: invite.role,
        }),
    ))
}
