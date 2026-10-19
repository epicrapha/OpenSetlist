//! Song CRUD, list/search, duplicate and export handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use super::model::{
    CreateSongRequest, ListQuery, Song, SongListResponse, UpdateSongRequest, SONG_COLUMNS,
};
use super::scope::WorkspaceScope;
use crate::error::AppError;
use crate::state::AppState;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;

/// Normalised list filters, built once and applied to both the page query and
/// the total-count query.
struct Filters {
    q: Option<String>,
    key: Option<String>,
    artist: Option<String>,
    tag: Option<String>,
}

fn apply_filters(qb: &mut QueryBuilder<'_, Postgres>, f: &Filters) {
    if let Some(q) = f.q.as_deref() {
        let pattern = format!("%{q}%");
        qb.push(" and (title ilike ")
            .push_bind(pattern.clone())
            .push(" or artist ilike ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(key) = f.key.as_deref() {
        qb.push(" and default_key = ").push_bind(key.to_string());
    }
    if let Some(artist) = f.artist.as_deref() {
        qb.push(" and artist ilike ")
            .push_bind(format!("%{artist}%"));
    }
    if let Some(tag) = f.tag.as_deref() {
        qb.push(" and ")
            .push_bind(tag.to_string())
            .push(" = any(tags)");
    }
}

/// GET /api/v1/songs
pub async fn list(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Query(params): Query<ListQuery>,
) -> Result<Json<SongListResponse>, AppError> {
    let order = match params.sort.as_deref() {
        None | Some("title") => " order by lower(title) asc, id asc",
        Some("updated_at") => " order by updated_at desc, id desc",
        Some(_) => {
            return Err(AppError::Validation(
                "sort must be title or updated_at".into(),
            ))
        }
    };
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);

    let filters = Filters {
        q: params.q.filter(|s| !s.trim().is_empty()),
        key: params.key.filter(|s| !s.trim().is_empty()),
        artist: params.artist.filter(|s| !s.trim().is_empty()),
        tag: params.tag.filter(|s| !s.trim().is_empty()),
    };

    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;

    let mut qb =
        QueryBuilder::<Postgres>::new(format!("select {SONG_COLUMNS} from songs where true"));
    apply_filters(&mut qb, &filters);
    qb.push(order);
    qb.push(" limit ").push_bind(limit);
    qb.push(" offset ").push_bind(offset);
    let songs = qb.build_query_as::<Song>().fetch_all(&mut *tx).await?;

    let mut count = QueryBuilder::<Postgres>::new("select count(*) from songs where true");
    apply_filters(&mut count, &filters);
    let total: i64 = count.build_query_scalar().fetch_one(&mut *tx).await?;

    tx.commit().await?;
    Ok(Json(SongListResponse { songs, total }))
}

/// POST /api/v1/songs
pub async fn create(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Json(req): Json<CreateSongRequest>,
) -> Result<(StatusCode, Json<Song>), AppError> {
    scope.require_leader()?;
    let title = req.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("title is required".into()));
    }

    let time_signature = req
        .time_signature
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "4/4".to_string());
    let tags = req.tags.unwrap_or_default();

    let sql = format!(
        "insert into songs (workspace_id, title, artist, default_key, tempo, time_signature, \
         tags, ccli_number, copyright, chordpro) values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) \
         returning {SONG_COLUMNS}"
    );
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let song = sqlx::query_as::<_, Song>(&sql)
        .bind(scope.workspace_id)
        .bind(title)
        .bind(req.artist.as_deref())
        .bind(req.default_key.as_deref())
        .bind(req.tempo)
        .bind(&time_signature)
        .bind(&tags)
        .bind(req.ccli_number.as_deref())
        .bind(req.copyright.as_deref())
        .bind(&req.chordpro)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(song)))
}

/// GET /api/v1/songs/:id
pub async fn get(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<Json<Song>, AppError> {
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let song = fetch_song(&mut tx, id).await?;
    Ok(Json(song))
}

/// PATCH /api/v1/songs/:id
pub async fn update(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSongRequest>,
) -> Result<Json<Song>, AppError> {
    scope.require_leader()?;
    if let Some(Some(title)) = &req.title {
        if title.trim().is_empty() {
            return Err(AppError::Validation("title must not be empty".into()));
        }
    }

    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let mut qb = QueryBuilder::<Postgres>::new("update songs set updated_at = now()");
    if let Some(title) = &req.title {
        qb.push(", title = ")
            .push_bind(title.as_ref().map(|t| t.trim().to_string()));
    }
    if let Some(artist) = &req.artist {
        qb.push(", artist = ").push_bind(artist.clone());
    }
    if let Some(key) = &req.default_key {
        qb.push(", default_key = ").push_bind(key.clone());
    }
    if let Some(tempo) = &req.tempo {
        qb.push(", tempo = ").push_bind(*tempo);
    }
    if let Some(sig) = &req.time_signature {
        qb.push(", time_signature = ").push_bind(sig.clone());
    }
    if let Some(tags) = &req.tags {
        qb.push(", tags = ").push_bind(tags.clone());
    }
    if let Some(ccli) = &req.ccli_number {
        qb.push(", ccli_number = ").push_bind(ccli.clone());
    }
    if let Some(copyright) = &req.copyright {
        qb.push(", copyright = ").push_bind(copyright.clone());
    }
    if let Some(chordpro) = &req.chordpro {
        qb.push(", chordpro = ").push_bind(chordpro.clone());
    }
    qb.push(" where id = ")
        .push_bind(id)
        .push(" returning ")
        .push(SONG_COLUMNS);

    let song = qb
        .build_query_as::<Song>()
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;
    tx.commit().await?;
    Ok(Json(song))
}

/// DELETE /api/v1/songs/:id
pub async fn delete(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let result = sqlx::query("delete from songs where id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await;
    let result = match result {
        // 23503 = foreign_key_violation. `setlist_items.song_id` is ON DELETE
        // RESTRICT: never silently drop a song out of a setlist.
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23503") => {
            return Err(AppError::Conflict(
                "song is referenced by a setlist; remove it from the setlist first".into(),
            ))
        }
        other => other?,
    };
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/songs/:id/duplicate
pub async fn duplicate(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Song>), AppError> {
    scope.require_leader()?;
    let sql = format!(
        "insert into songs (id, workspace_id, title, artist, default_key, tempo, time_signature, \
         tags, ccli_number, copyright, chordpro) \
         select $1, workspace_id, title || ' (copy)', artist, default_key, tempo, time_signature, \
         tags, ccli_number, copyright, chordpro from songs where id = $2 returning {SONG_COLUMNS}"
    );
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let song = sqlx::query_as::<_, Song>(&sql)
        .bind(Uuid::new_v4())
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(song)))
}

#[derive(Debug, serde::Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

/// GET /api/v1/songs/:id/export?format=chordpro|text
pub async fn export(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let song = fetch_song(&mut tx, id).await?;

    let (body, extension) = match query.format.as_deref().unwrap_or("chordpro") {
        "chordpro" => (song.chordpro.clone(), "cho"),
        "text" => (chordpro_to_text(&song.chordpro), "txt"),
        _ => {
            return Err(AppError::Validation(
                "format must be chordpro or text".into(),
            ))
        }
    };

    let filename = format!("{}.{extension}", sanitize_filename(&song.title));
    let headers = [
        (
            header::CONTENT_TYPE,
            "text/plain; charset=utf-8".to_string(),
        ),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        ),
    ];
    Ok((headers, body).into_response())
}

async fn fetch_song(tx: &mut crate::tenant::TenantTx<'_>, id: Uuid) -> Result<Song, AppError> {
    let sql = format!("select {SONG_COLUMNS} from songs where id = $1");
    sqlx::query_as::<_, Song>(&sql)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::NotFound)
}

/// Strip ChordPro directives (`{...}` lines) and inline chord markers (`[C]`)
/// to produce plain lyrics.
fn chordpro_to_text(input: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            continue;
        }
        let mut text = String::new();
        let mut in_chord = false;
        for ch in line.chars() {
            match ch {
                '[' if !in_chord => in_chord = true,
                ']' if in_chord => in_chord = false,
                _ if !in_chord => text.push(ch),
                _ => {}
            }
        }
        out.push_str(text.trim_end());
        out.push('\n');
    }
    out
}

fn sanitize_filename(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let cleaned = cleaned.trim_matches('_');
    if cleaned.is_empty() {
        "song".to_string()
    } else {
        cleaned.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{chordpro_to_text, sanitize_filename};

    #[test]
    fn plain_text_drops_directives_and_chords() {
        let input = "{title: Amazing Grace}\n[G]Amazing [C]grace\n{comment: verse}\nhow [D]sweet";
        let text = chordpro_to_text(input);
        assert_eq!(text, "Amazing grace\nhow sweet\n");
    }

    #[test]
    fn filename_is_sanitised() {
        assert_eq!(sanitize_filename("Amazing Grace!"), "Amazing_Grace");
        assert_eq!(sanitize_filename("///"), "song");
    }
}
