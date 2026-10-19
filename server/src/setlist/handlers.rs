//! Setlist HTTP handlers (PLAN.md §3.3).
//!
//! Reads need any membership (viewer+); every write calls
//! `WorkspaceScope::require_leader`. The public share endpoint runs with no
//! session/tenant, under a token-scoped RLS transaction.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use super::model::{
    CreateItemRequest, CreateSetlistRequest, ItemsResponse, ListSetlistsQuery, ReorderRequest,
    Setlist, SetlistDetailResponse, SetlistItem, SetlistItemRow, SetlistListResponse,
    ShareResponse, UpdateItemRequest, UpdateSetlistRequest, ITEM_COLUMNS, SETLIST_COLUMNS,
};
use crate::error::AppError;
use crate::song::scope::WorkspaceScope;
use crate::state::AppState;
use crate::tenant::TenantTx;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;
const STATUSES: [&str; 3] = ["draft", "ready", "archived"];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn fetch_setlist(tx: &mut TenantTx<'_>, id: Uuid) -> Result<Setlist, AppError> {
    let sql = format!("select {SETLIST_COLUMNS} from setlists s where s.id = $1");
    sqlx::query_as::<_, Setlist>(&sql)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::NotFound)
}

async fn fetch_item_row(
    tx: &mut TenantTx<'_>,
    setlist_id: Uuid,
    item_id: Uuid,
) -> Result<SetlistItemRow, AppError> {
    let sql = format!(
        "select {ITEM_COLUMNS} from setlist_items si \
         left join songs s on s.id = si.song_id \
         where si.setlist_id = $1 and si.id = $2"
    );
    sqlx::query_as::<_, SetlistItemRow>(&sql)
        .bind(setlist_id)
        .bind(item_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::NotFound)
}

async fn fetch_item(
    tx: &mut TenantTx<'_>,
    setlist_id: Uuid,
    item_id: Uuid,
) -> Result<SetlistItem, AppError> {
    Ok(fetch_item_row(tx, setlist_id, item_id).await?.into())
}

async fn fetch_items(
    tx: &mut TenantTx<'_>,
    setlist_id: Uuid,
) -> Result<Vec<SetlistItem>, AppError> {
    let sql = format!(
        "select {ITEM_COLUMNS} from setlist_items si \
         left join songs s on s.id = si.song_id \
         where si.setlist_id = $1 order by si.position asc, si.id asc"
    );
    let rows = sqlx::query_as::<_, SetlistItemRow>(&sql)
        .bind(setlist_id)
        .fetch_all(&mut **tx)
        .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

fn ensure_not_archived(setlist: &Setlist) -> Result<(), AppError> {
    if setlist.status == "archived" {
        Err(AppError::Conflict("setlist is archived".into()))
    } else {
        Ok(())
    }
}

async fn touch_setlist(tx: &mut TenantTx<'_>, id: Uuid) -> Result<(), AppError> {
    sqlx::query("update setlists set updated_at = now() where id = $1")
        .bind(id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn rewrite_order(
    tx: &mut TenantTx<'_>,
    setlist_id: Uuid,
    ids: &[Uuid],
) -> Result<(), AppError> {
    for (position, id) in ids.iter().enumerate() {
        sqlx::query("update setlist_items set position = $1 where setlist_id = $2 and id = $3")
            .bind(position as i32)
            .bind(setlist_id)
            .bind(id)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn move_item(
    tx: &mut TenantTx<'_>,
    setlist_id: Uuid,
    item_id: Uuid,
    target: i32,
) -> Result<(), AppError> {
    let mut ids: Vec<Uuid> = sqlx::query_scalar(
        "select id from setlist_items where setlist_id = $1 order by position asc, id asc",
    )
    .bind(setlist_id)
    .fetch_all(&mut **tx)
    .await?;
    if let Some(current) = ids.iter().position(|id| *id == item_id) {
        ids.remove(current);
    }
    let target = target.clamp(0, ids.len() as i32) as usize;
    ids.insert(target, item_id);
    rewrite_order(tx, setlist_id, &ids).await
}

fn apply_filters(qb: &mut QueryBuilder<'_, Postgres>, status: &Option<String>, q: &Option<String>) {
    if let Some(status) = status {
        qb.push(" and s.status = ").push_bind(status.clone());
    }
    if let Some(q) = q {
        qb.push(" and s.title ilike ").push_bind(format!("%{q}%"));
    }
}

// ---------------------------------------------------------------------------
// Setlist CRUD
// ---------------------------------------------------------------------------

/// GET /api/v1/setlists
pub async fn list(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Query(params): Query<ListSetlistsQuery>,
) -> Result<Json<SetlistListResponse>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let status = params.status.filter(|s| !s.trim().is_empty());
    let q = params.q.filter(|s| !s.trim().is_empty());

    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;

    let mut qb = QueryBuilder::<Postgres>::new(format!(
        "select {SETLIST_COLUMNS} from setlists s where true"
    ));
    apply_filters(&mut qb, &status, &q);
    qb.push(" order by s.updated_at desc, s.id desc limit ")
        .push_bind(limit)
        .push(" offset ")
        .push_bind(offset);
    let setlists = qb.build_query_as::<Setlist>().fetch_all(&mut *tx).await?;

    let mut count = QueryBuilder::<Postgres>::new("select count(*) from setlists s where true");
    apply_filters(&mut count, &status, &q);
    let total: i64 = count.build_query_scalar().fetch_one(&mut *tx).await?;

    tx.commit().await?;
    Ok(Json(SetlistListResponse { setlists, total }))
}

/// POST /api/v1/setlists
pub async fn create(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Json(req): Json<CreateSetlistRequest>,
) -> Result<(StatusCode, Json<Setlist>), AppError> {
    scope.require_leader()?;
    let title = req.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("title is required".into()));
    }

    let id = Uuid::new_v4();
    let mut tx = state
        .scoped(Some(scope.workspace_id), Some(scope.user_id))
        .await?;
    sqlx::query(
        "insert into setlists (id, workspace_id, title, service_date, service_type, created_by) \
         values ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(scope.workspace_id)
    .bind(title)
    .bind(req.service_date)
    .bind(req.service_type.as_deref())
    .bind(scope.user_id)
    .execute(&mut *tx)
    .await?;

    let setlist = fetch_setlist(&mut tx, id).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(setlist)))
}

/// GET /api/v1/setlists/:id
pub async fn get(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<Json<SetlistDetailResponse>, AppError> {
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let setlist = fetch_setlist(&mut tx, id).await?;
    let items = fetch_items(&mut tx, id).await?;
    Ok(Json(SetlistDetailResponse { setlist, items }))
}

/// PATCH /api/v1/setlists/:id
pub async fn update(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSetlistRequest>,
) -> Result<Json<Setlist>, AppError> {
    scope.require_leader()?;

    if let Some(Some(title)) = &req.title {
        if title.trim().is_empty() {
            return Err(AppError::Validation("title must not be empty".into()));
        }
    } else if matches!(req.title, Some(None)) {
        return Err(AppError::Validation("title must not be null".into()));
    }

    match &req.status {
        Some(Some(status)) if STATUSES.contains(&status.as_str()) => {}
        Some(_) => {
            return Err(AppError::Validation(
                "status must be one of draft, ready, archived".into(),
            ))
        }
        None => {}
    }

    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let _existing = fetch_setlist(&mut tx, id).await?;

    let mut qb = QueryBuilder::<Postgres>::new("update setlists set updated_at = now()");
    if let Some(Some(title)) = &req.title {
        qb.push(", title = ").push_bind(title.trim().to_string());
    }
    if let Some(service_date) = &req.service_date {
        qb.push(", service_date = ").push_bind(*service_date);
    }
    if let Some(service_type) = &req.service_type {
        qb.push(", service_type = ").push_bind(service_type.clone());
    }
    if let Some(Some(status)) = &req.status {
        qb.push(", status = ").push_bind(status.clone());
    }
    if let Some(is_template) = req.is_template {
        qb.push(", is_template = ").push_bind(is_template);
    }
    qb.push(" where id = ").push_bind(id);
    qb.build().execute(&mut *tx).await?;

    let setlist = fetch_setlist(&mut tx, id).await?;
    tx.commit().await?;
    Ok(Json(setlist))
}

/// DELETE /api/v1/setlists/:id
pub async fn delete(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let result = sqlx::query("delete from setlists where id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/setlists/:id/duplicate
pub async fn duplicate(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Setlist>), AppError> {
    scope.require_leader()?;
    let mut tx = state
        .scoped(Some(scope.workspace_id), Some(scope.user_id))
        .await?;
    let source = fetch_setlist(&mut tx, id).await?;

    let new_id = Uuid::new_v4();
    sqlx::query(
        "insert into setlists (id, workspace_id, title, service_date, service_type, status, \
         is_template, created_by) values ($1, $2, $3, $4, $5, 'draft', false, $6)",
    )
    .bind(new_id)
    .bind(scope.workspace_id)
    .bind(format!("{} (copy)", source.title))
    .bind(source.service_date)
    .bind(source.service_type.as_deref())
    .bind(scope.user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "insert into setlist_items (workspace_id, setlist_id, position, kind, song_id, title, \
         notes, key_override, capo_override, duration_seconds) \
         select $1, $2, position, kind, song_id, title, notes, key_override, capo_override, \
         duration_seconds from setlist_items where setlist_id = $3",
    )
    .bind(scope.workspace_id)
    .bind(new_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    let setlist = fetch_setlist(&mut tx, new_id).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(setlist)))
}

// ---------------------------------------------------------------------------
// Items
// ---------------------------------------------------------------------------

/// POST /api/v1/setlists/:id/items
pub async fn create_item(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(setlist_id): Path<Uuid>,
    Json(req): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<SetlistItem>), AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let setlist = fetch_setlist(&mut tx, setlist_id).await?;
    ensure_not_archived(&setlist)?;

    let kind = req.kind.as_str();
    let (song_id, title) = match kind {
        "song" => {
            let song_id = req.song_id.ok_or(AppError::NotFound)?;
            crate::song::ensure_song(&mut tx, song_id).await?;
            (Some(song_id), req.title)
        }
        "note" => {
            let title = req
                .title
                .as_deref()
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .map(str::to_string)
                .ok_or_else(|| AppError::BadRequest("title is required for note items".into()))?;
            (req.song_id, Some(title))
        }
        _ => return Err(AppError::Validation("kind must be song or note".into())),
    };

    let count: i64 = sqlx::query_scalar("select count(*) from setlist_items where setlist_id = $1")
        .bind(setlist_id)
        .fetch_one(&mut *tx)
        .await?;
    let position = req.position.unwrap_or(count as i32).clamp(0, count as i32);
    if position < count as i32 {
        sqlx::query(
            "update setlist_items set position = position + 1 \
             where setlist_id = $1 and position >= $2",
        )
        .bind(setlist_id)
        .bind(position)
        .execute(&mut *tx)
        .await?;
    }

    let item_id = Uuid::new_v4();
    sqlx::query(
        "insert into setlist_items (id, workspace_id, setlist_id, position, kind, song_id, title, \
         notes, key_override, capo_override, duration_seconds) \
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(item_id)
    .bind(scope.workspace_id)
    .bind(setlist_id)
    .bind(position)
    .bind(kind)
    .bind(song_id)
    .bind(title.as_deref())
    .bind(req.notes.as_deref())
    .bind(req.key_override.as_deref())
    .bind(req.capo_override)
    .bind(req.duration_seconds)
    .execute(&mut *tx)
    .await?;

    touch_setlist(&mut tx, setlist_id).await?;
    let item = fetch_item(&mut tx, setlist_id, item_id).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// PATCH /api/v1/setlists/:id/items/:item_id
pub async fn update_item(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path((setlist_id, item_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateItemRequest>,
) -> Result<Json<SetlistItem>, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let setlist = fetch_setlist(&mut tx, setlist_id).await?;
    ensure_not_archived(&setlist)?;
    let existing = fetch_item_row(&mut tx, setlist_id, item_id).await?;

    // Validate the item's resulting state, not just the supplied fields.
    let kind = match &req.kind {
        Some(Some(kind)) => kind.clone(),
        Some(None) => return Err(AppError::Validation("kind must not be null".into())),
        None => existing.kind.clone(),
    };
    if kind != "song" && kind != "note" {
        return Err(AppError::Validation("kind must be song or note".into()));
    }
    let resulting_song = match &req.song_id {
        Some(song_id) => *song_id,
        None => existing.song_id,
    };
    let resulting_title = match &req.title {
        Some(title) => title.clone(),
        None => existing.title.clone(),
    };
    match kind.as_str() {
        "song" => {
            let song_id = resulting_song.ok_or(AppError::NotFound)?;
            crate::song::ensure_song(&mut tx, song_id).await?;
        }
        "note" => {
            if resulting_title
                .as_deref()
                .map(|title| title.trim().is_empty())
                .unwrap_or(true)
            {
                return Err(AppError::BadRequest(
                    "title is required for note items".into(),
                ));
            }
        }
        _ => unreachable!(),
    }

    let mut qb = QueryBuilder::<Postgres>::new("update setlist_items set id = id");
    if let Some(Some(kind)) = &req.kind {
        qb.push(", kind = ").push_bind(kind.clone());
    }
    if let Some(song_id) = &req.song_id {
        qb.push(", song_id = ").push_bind(*song_id);
    }
    if let Some(title) = &req.title {
        qb.push(", title = ")
            .push_bind(title.as_ref().map(|title| title.trim().to_string()));
    }
    if let Some(notes) = &req.notes {
        qb.push(", notes = ").push_bind(notes.clone());
    }
    if let Some(key_override) = &req.key_override {
        qb.push(", key_override = ").push_bind(key_override.clone());
    }
    if let Some(capo_override) = &req.capo_override {
        qb.push(", capo_override = ").push_bind(*capo_override);
    }
    if let Some(duration) = &req.duration_seconds {
        qb.push(", duration_seconds = ").push_bind(*duration);
    }
    qb.push(" where setlist_id = ")
        .push_bind(setlist_id)
        .push(" and id = ")
        .push_bind(item_id);
    qb.build().execute(&mut *tx).await?;

    if let Some(target) = req.position {
        move_item(&mut tx, setlist_id, item_id, target).await?;
    }

    touch_setlist(&mut tx, setlist_id).await?;
    let item = fetch_item(&mut tx, setlist_id, item_id).await?;
    tx.commit().await?;
    Ok(Json(item))
}

/// DELETE /api/v1/setlists/:id/items/:item_id
pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path((setlist_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let setlist = fetch_setlist(&mut tx, setlist_id).await?;
    ensure_not_archived(&setlist)?;

    let position: Option<i32> =
        sqlx::query_scalar("select position from setlist_items where setlist_id = $1 and id = $2")
            .bind(setlist_id)
            .bind(item_id)
            .fetch_optional(&mut *tx)
            .await?;
    let position = position.ok_or(AppError::NotFound)?;

    sqlx::query("delete from setlist_items where setlist_id = $1 and id = $2")
        .bind(setlist_id)
        .bind(item_id)
        .execute(&mut *tx)
        .await?;

    // Items were dense, so everything after the removed slot shifts down one.
    sqlx::query(
        "update setlist_items set position = position - 1 \
         where setlist_id = $1 and position > $2",
    )
    .bind(setlist_id)
    .bind(position)
    .execute(&mut *tx)
    .await?;

    touch_setlist(&mut tx, setlist_id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/v1/setlists/:id/items/order
pub async fn reorder(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(setlist_id): Path<Uuid>,
    Json(req): Json<ReorderRequest>,
) -> Result<Json<ItemsResponse>, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let setlist = fetch_setlist(&mut tx, setlist_id).await?;
    ensure_not_archived(&setlist)?;

    let mut existing: Vec<Uuid> =
        sqlx::query_scalar("select id from setlist_items where setlist_id = $1")
            .bind(setlist_id)
            .fetch_all(&mut *tx)
            .await?;
    let mut provided = req.item_ids.clone();
    existing.sort_unstable();
    provided.sort_unstable();
    if existing != provided {
        return Err(AppError::Validation(
            "item_ids must contain exactly the setlist's items".into(),
        ));
    }

    rewrite_order(&mut tx, setlist_id, &req.item_ids).await?;
    touch_setlist(&mut tx, setlist_id).await?;
    let items = fetch_items(&mut tx, setlist_id).await?;
    tx.commit().await?;
    Ok(Json(ItemsResponse { items }))
}

// ---------------------------------------------------------------------------
// Sharing
// ---------------------------------------------------------------------------

/// POST /api/v1/setlists/:id/share — idempotent: creates the token once.
pub async fn create_share(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<Json<ShareResponse>, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let setlist = fetch_setlist(&mut tx, id).await?;
    ensure_not_archived(&setlist)?;

    if let Some(token) = setlist.share_token {
        tx.commit().await?;
        return Ok(Json(ShareResponse { share_token: token }));
    }

    let token = crate::token::random_token();
    sqlx::query("update setlists set share_token = $1, updated_at = now() where id = $2")
        .bind(&token)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(ShareResponse { share_token: token }))
}

/// DELETE /api/v1/setlists/:id/share — revokes the token.
pub async fn revoke_share(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let _existing = fetch_setlist(&mut tx, id).await?;
    sqlx::query("update setlists set share_token = null, updated_at = now() where id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/shared/setlists/:token — public, no session or workspace header.
/// Runs as `opensetlist_app` with only `app.current_share_token` set, so RLS
/// exposes exactly the shared setlist, its items, and those items' songs.
pub async fn shared(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Json<SetlistDetailResponse>, AppError> {
    if token.trim().is_empty() {
        return Err(AppError::NotFound);
    }

    let mut tx = state.scoped_share_token(&token).await?;
    let sql = format!("select {SETLIST_COLUMNS} from setlists s where s.share_token = $1");
    let setlist = sqlx::query_as::<_, Setlist>(&sql)
        .bind(&token)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;
    let items = fetch_items(&mut tx, setlist.id).await?;
    tx.commit().await?;
    Ok(Json(SetlistDetailResponse { setlist, items }))
}
