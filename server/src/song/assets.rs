//! Song asset handlers: reference links (SONG-6) and PDF uploads (SONG-7).

use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use super::model::{Asset, AssetListResponse, CreateLinkRequest, ASSET_COLUMNS};
use super::scope::WorkspaceScope;
use crate::error::AppError;
use crate::state::AppState;

/// 10 MB cap (contract SONG-7).
pub const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

/// GET /api/v1/songs/:id/assets — list a song's links and uploads.
pub async fn list(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(song_id): Path<Uuid>,
) -> Result<Json<AssetListResponse>, AppError> {
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    super::ensure_song(&mut tx, song_id).await?;

    let sql = format!(
        "select {ASSET_COLUMNS} from song_assets where song_id = $1 \
         order by created_at asc, id asc"
    );
    let assets = sqlx::query_as::<_, Asset>(&sql)
        .bind(song_id)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(AssetListResponse { assets }))
}

/// POST /api/v1/songs/:id/assets — attach a reference link.
pub async fn create_link(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(song_id): Path<Uuid>,
    Json(req): Json<CreateLinkRequest>,
) -> Result<(StatusCode, Json<Asset>), AppError> {
    scope.require_leader()?;
    if req.kind != "link" {
        return Err(AppError::Validation(
            "kind must be \"link\"; use /assets/pdf for uploads".into(),
        ));
    }
    let url = req.url.trim();
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(AppError::Validation("url must be an http(s) URL".into()));
    }

    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    super::ensure_song(&mut tx, song_id).await?;

    let sql = format!(
        "insert into song_assets (workspace_id, song_id, kind, url, label) \
         values ($1,$2,'link',$3,$4) returning {ASSET_COLUMNS}"
    );
    let asset = sqlx::query_as::<_, Asset>(&sql)
        .bind(scope.workspace_id)
        .bind(song_id)
        .bind(url)
        .bind(req.label.as_deref())
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(asset)))
}

/// POST /api/v1/songs/:id/assets/pdf — upload a PDF (multipart field `file`).
pub async fn upload_pdf(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(song_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Asset>), AppError> {
    scope.require_leader()?;

    let field = loop {
        match multipart.next_field().await.map_err(multipart_error)? {
            Some(field) if field.name() == Some("file") => break field,
            Some(_) => continue,
            None => {
                return Err(AppError::Validation(
                    "multipart field 'file' is required".into(),
                ))
            }
        }
    };

    let content_type = field.content_type().unwrap_or_default().to_string();
    let filename = field.file_name().map(str::to_string);
    let data = field.bytes().await.map_err(multipart_error)?;

    if data.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::PayloadTooLarge(
            "file exceeds the 10 MB limit".into(),
        ));
    }
    if !content_type.eq_ignore_ascii_case("application/pdf") {
        return Err(AppError::UnsupportedMedia(
            "only application/pdf uploads are accepted".into(),
        ));
    }
    // Content-Type is client-controlled; verify the actual PDF signature too.
    if !data.starts_with(b"%PDF-") {
        return Err(AppError::UnsupportedMedia("file is not a valid PDF".into()));
    }

    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    super::ensure_song(&mut tx, song_id).await?;

    // Random object name; the client filename is only ever stored as metadata.
    let object_name = format!("{}.pdf", Uuid::new_v4().simple());
    tokio::fs::create_dir_all(&state.config.upload_dir)
        .await
        .map_err(AppError::internal)?;
    tokio::fs::write(state.config.upload_dir.join(&object_name), &data)
        .await
        .map_err(AppError::internal)?;

    let sql = format!(
        "insert into song_assets (workspace_id, song_id, kind, filename, size_bytes, \
         content_type, object_name) values ($1,$2,'pdf',$3,$4,$5,$6) returning {ASSET_COLUMNS}"
    );
    let asset = sqlx::query_as::<_, Asset>(&sql)
        .bind(scope.workspace_id)
        .bind(song_id)
        .bind(filename)
        .bind(data.len() as i64)
        .bind("application/pdf")
        .bind(&object_name)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(asset)))
}

/// GET /api/v1/assets/:id/file — download an uploaded PDF.
pub async fn download(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let sql = format!("select {ASSET_COLUMNS} from song_assets where id = $1");
    let asset = sqlx::query_as::<_, Asset>(&sql)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;
    tx.commit().await?;

    if asset.kind != "pdf" {
        return Err(AppError::NotFound);
    }
    let object_name = asset.object_name.ok_or(AppError::NotFound)?;
    let path = state.config.upload_dir.join(&object_name);

    // Defence in depth: the object name comes from our own random generator, but
    // never trust a stored path blindly — resolve and confirm it stays inside the
    // upload root.
    let base = tokio::fs::canonicalize(&state.config.upload_dir)
        .await
        .map_err(AppError::internal)?;
    let resolved = tokio::fs::canonicalize(&path)
        .await
        .map_err(|_| AppError::NotFound)?;
    if !resolved.starts_with(&base) {
        return Err(AppError::NotFound);
    }

    let bytes = tokio::fs::read(&resolved)
        .await
        .map_err(AppError::internal)?;
    let filename = asset.filename.unwrap_or_else(|| "file.pdf".to_string());
    let filename = filename.replace(['"', '\r', '\n'], "_");
    let headers = [
        (header::CONTENT_TYPE, "application/pdf".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        ),
    ];
    Ok((headers, bytes).into_response())
}

/// DELETE /api/v1/assets/:id
pub async fn delete(
    State(state): State<Arc<AppState>>,
    scope: WorkspaceScope,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    scope.require_leader()?;
    let mut tx = state.scoped(Some(scope.workspace_id), None).await?;
    let sql = format!("select {ASSET_COLUMNS} from song_assets where id = $1");
    let asset = sqlx::query_as::<_, Asset>(&sql)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;

    sqlx::query("delete from song_assets where id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    if let Some(object_name) = asset.object_name {
        if let Err(error) = tokio::fs::remove_file(state.config.upload_dir.join(&object_name)).await
        {
            tracing::warn!(%error, "failed to remove asset file from disk");
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

fn multipart_error(error: axum::extract::multipart::MultipartError) -> AppError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::PayloadTooLarge("upload exceeds the 10 MB limit".into())
    } else {
        AppError::Validation(format!("invalid multipart body: {error}"))
    }
}
