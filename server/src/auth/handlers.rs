//! Auth HTTP handlers: register, login, logout, me.

use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tower_sessions::Session;

use super::model::{LoginRequest, MeResponse, MembershipView, RegisterRequest, User, UserResponse};
use super::AuthUser;
use crate::error::AppError;
use crate::state::AppState;

const USER_COLUMNS: &str = "id, email::text as email, name, password_hash, created_at";

fn validate_email(email: &str) -> Result<(), AppError> {
    let email = email.trim();
    if email.len() < 3 || !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(AppError::Validation("a valid email is required".into()));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    validate_email(&req.email)?;
    validate_password(&req.password)?;
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }

    let password_hash = super::password::hash(req.password).await?;

    let sql = format!(
        "insert into users (email, name, password_hash) values ($1::citext, $2, $3) \
         on conflict (email) do nothing returning {USER_COLUMNS}"
    );
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(req.email.trim())
        .bind(req.name.trim())
        .bind(&password_hash)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::Conflict("email already registered".into()))?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    session: Session,
    Json(req): Json<LoginRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let sql = format!("select {USER_COLUMNS} from users where email = $1::citext");
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(req.email.trim())
        .fetch_optional(&state.pool)
        .await?;

    // Always run a verify (against a dummy hash when the user is unknown) so the
    // response time does not reveal whether the account exists.
    let hash = user
        .as_ref()
        .map(|u| u.password_hash.clone())
        .unwrap_or_else(|| state.dummy_password_hash.clone());
    let valid = super::password::verify(req.password, hash).await?;

    let user = match (user, valid) {
        (Some(user), true) => user,
        _ => return Err(AppError::Unauthorized),
    };

    // Rotate the session id on login to prevent session fixation.
    session.cycle_id().await.map_err(AppError::from)?;
    session
        .insert("user_id", user.id)
        .await
        .map_err(AppError::from)?;

    Ok(Json(user.into()))
}

/// POST /api/v1/auth/logout
pub async fn logout(session: Session) -> Result<StatusCode, AppError> {
    session.flush().await.map_err(AppError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/auth/me
pub async fn me(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<MeResponse>, AppError> {
    let AuthUser(user_id) = user;

    let sql = format!("select {USER_COLUMNS} from users where id = $1");
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    // Memberships are RLS-protected. The user-scoped policy lets a user read
    // their own rows across every workspace without a tenant setting.
    let mut tx = state.scoped(None, Some(user_id)).await?;
    let memberships = sqlx::query_as::<_, MembershipView>(
        "select workspace_id, role from memberships where user_id = $1 order by workspace_id",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    Ok(Json(MeResponse {
        user: user.into(),
        memberships,
    }))
}
