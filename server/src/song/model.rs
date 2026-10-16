//! Song library models, matching the frozen contract in PLAN.md §3.1.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A song. Doubles as the JSON response (fields match the contract exactly) and
/// the `FromRow` target, so there is no separate response type to keep in sync.
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Song {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub default_key: Option<String>,
    pub tempo: Option<i32>,
    pub time_signature: String,
    pub tags: Vec<String>,
    pub ccli_number: Option<String>,
    pub copyright: Option<String>,
    pub chordpro: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `pub` so tests/queries can `select {COLUMNS} from songs`.
pub const SONG_COLUMNS: &str = "id, workspace_id, title, artist, default_key, tempo, \
     time_signature, tags, ccli_number, copyright, chordpro, created_at, updated_at";

#[derive(Debug, Deserialize)]
pub struct CreateSongRequest {
    pub title: String,
    pub artist: Option<String>,
    pub default_key: Option<String>,
    pub tempo: Option<i32>,
    pub time_signature: Option<String>,
    pub tags: Option<Vec<String>>,
    pub ccli_number: Option<String>,
    pub copyright: Option<String>,
    pub chordpro: String,
}

/// Partial update. `Option<Option<T>>` distinguishes "field absent" (`None`,
/// leave unchanged) from "field explicitly null" (`Some(None)`, clear it), which
/// a plain `Option<T>` cannot express.
#[derive(Debug, Default, Deserialize)]
pub struct UpdateSongRequest {
    #[serde(default, deserialize_with = "double_option")]
    pub title: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub artist: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub default_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub tempo: Option<Option<i32>>,
    #[serde(default, deserialize_with = "double_option")]
    pub time_signature: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub tags: Option<Option<Vec<String>>>,
    #[serde(default, deserialize_with = "double_option")]
    pub ccli_number: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub copyright: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub chordpro: Option<Option<String>>,
}

/// Wraps the normally-generated deserializer so an absent field stays `None`
/// while an explicit `null` becomes `Some(None)`.
pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Deserialize::deserialize(de).map(Some)
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub q: Option<String>,
    pub key: Option<String>,
    pub artist: Option<String>,
    pub tag: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SongListResponse {
    pub songs: Vec<Song>,
    pub total: i64,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Arrangement {
    pub id: Uuid,
    pub song_id: Uuid,
    pub name: String,
    pub key: Option<String>,
    pub chordpro: String,
}

pub const ARRANGEMENT_COLUMNS: &str = "id, song_id, name, key, chordpro";

#[derive(Debug, Deserialize)]
pub struct CreateArrangementRequest {
    pub name: String,
    pub key: Option<String>,
    pub chordpro: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArrangementRequest {
    #[serde(default, deserialize_with = "double_option")]
    pub name: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub key: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub chordpro: Option<Option<String>>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Asset {
    pub id: Uuid,
    pub song_id: Uuid,
    pub kind: String,
    pub url: Option<String>,
    pub label: Option<String>,
    pub filename: Option<String>,
    pub size_bytes: Option<i64>,
    pub content_type: Option<String>,
    /// On-disk object name — internal only, never serialised.
    #[serde(skip)]
    pub object_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub const ASSET_COLUMNS: &str = "id, song_id, kind, url, label, filename, size_bytes, \
     content_type, object_name, created_at";

#[derive(Debug, Serialize)]
pub struct AssetListResponse {
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub kind: String,
    pub url: String,
    pub label: Option<String>,
}
