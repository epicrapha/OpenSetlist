//! Song arrangement handlers (SONG-8).

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use super::model::{
    Arrangement, CreateArrangementRequest, UpdateArrangementRequest, ARRANGEMENT_COLUMNS,
};
use super::scope::WorkspaceScope;
use crate::error::AppError;
use crate::state::AppState;

/// GET /api/v1/songs/:id/arrangements
pub async fn list(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(song_id): Path<Uuid>,
) -> Result<Json<Vec<Arrangement>>, AppError> {
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    super::ensure_song(&mut tx, song_id).await?;
    let sql = format!(
        "select {ARRANGEMENT_COLUMNS} from song_arrangements where song_id = $1 order by name asc"
    );
    let arrangements = sqlx::query_as::<_, Arrangement>(&sql)
        .bind(song_id)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(arrangements))
}

/// POST /api/v1/songs/:id/arrangements
pub async fn create(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(song_id): Path<Uuid>,
    Json(req): Json<CreateArrangementRequest>,
) -> Result<(StatusCode, Json<Arrangement>), AppError> {
    scope.require_leader()?;
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }

    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    super::ensure_song(&mut tx, song_id).await?;

    let sql = format!(
        "insert into song_arrangements (workspace_id, song_id, name, key, chordpro) \
         values ($1,$2,$3,$4,$5) returning {ARRANGEMENT_COLUMNS}"
    );
    let arrangement = sqlx::query_as::<_, Arrangement>(&sql)
        .bind(scope.workspace_id)
        .bind(song_id)
        .bind(name)
        .bind(req.key.as_deref())
        .bind(req.chordpro.unwrap_or_default())
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(arrangement)))
}

/// PATCH /api/v1/arrangements/:id
pub async fn update(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateArrangementRequest>,
) -> Result<Json<Arrangement>, AppError> {
    scope.require_leader()?;
    if let Some(Some(name)) = &req.name {
        if name.trim().is_empty() {
            return Err(AppError::Validation("name must not be empty".into()));
        }
    }

    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let mut qb = QueryBuilder::<Postgres>::new("update song_arrangements set id = id");
    if let Some(name) = &req.name {
        qb.push(", name = ")
            .push_bind(name.as_ref().map(|n| n.trim().to_string()));
    }
    if let Some(key) = &req.key {
        qb.push(", key = ").push_bind(key.clone());
    }
    if let Some(chordpro) = &req.chordpro {
        qb.push(", chordpro = ").push_bind(chordpro.clone());
    }
    qb.push(" where id = ")
        .push_bind(id)
        .push(" returning ")
        .push(ARRANGEMENT_COLUMNS);

    let arrangement = qb
        .build_query_as::<Arrangement>()
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;
    tx.commit().await?;
    Ok(Json(arrangement))
}

/// DELETE /api/v1/arrangements/:id
pub async fn delete(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let result = sqlx::query("delete from song_arrangements where id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
