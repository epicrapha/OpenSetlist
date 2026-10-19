//! Setlist models, matching the frozen contract in PLAN.md §3.3.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::song::model::double_option;

/// Setlist header. Doubles as the JSON response and the `FromRow` target.
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Setlist {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub service_date: Option<NaiveDate>,
    pub service_type: Option<String>,
    pub status: String,
    pub is_template: bool,
    pub share_token: Option<String>,
    pub item_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Always query `from setlists s`; the correlated count is RLS-checked too.
pub const SETLIST_COLUMNS: &str = "s.id, s.workspace_id, s.title, s.service_date, \
     s.service_type, s.status, s.is_template, s.share_token, \
     (select count(*) from setlist_items si where si.setlist_id = s.id)::int as item_count, \
     s.created_at, s.updated_at";

#[derive(Debug, Deserialize)]
pub struct CreateSetlistRequest {
    pub title: String,
    pub service_date: Option<NaiveDate>,
    pub service_type: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct UpdateSetlistRequest {
    #[serde(default, deserialize_with = "double_option")]
    pub title: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub service_date: Option<Option<NaiveDate>>,
    #[serde(default, deserialize_with = "double_option")]
    pub service_type: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub status: Option<Option<String>>,
    pub is_template: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListSetlistsQuery {
    pub status: Option<String>,
    pub q: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SetlistListResponse {
    pub setlists: Vec<Setlist>,
    pub total: i64,
}

/// The embedded song on an item read — only the fields the contract exposes.
#[derive(Debug, Serialize)]
pub struct SongSummary {
    pub id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub default_key: Option<String>,
    pub tempo: Option<i32>,
    pub time_signature: String,
    pub chordpro: String,
}

/// Flat row produced by the item query (item columns + aliased song columns).
#[derive(Debug, sqlx::FromRow)]
pub struct SetlistItemRow {
    pub id: Uuid,
    pub setlist_id: Uuid,
    pub position: i32,
    pub kind: String,
    pub song_id: Option<Uuid>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub key_override: Option<String>,
    pub capo_override: Option<i16>,
    pub duration_seconds: Option<i32>,
    pub song_title: Option<String>,
    pub song_artist: Option<String>,
    pub song_default_key: Option<String>,
    pub song_tempo: Option<i32>,
    pub song_time_signature: Option<String>,
    pub song_chordpro: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetlistItem {
    pub id: Uuid,
    pub setlist_id: Uuid,
    pub position: i32,
    pub kind: String,
    pub song_id: Option<Uuid>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub key_override: Option<String>,
    pub capo_override: Option<i16>,
    pub duration_seconds: Option<i32>,
    pub song: Option<SongSummary>,
}

impl From<SetlistItemRow> for SetlistItem {
    fn from(row: SetlistItemRow) -> Self {
        // song_title is NOT NULL on songs, so its presence means the LEFT JOIN
        // matched (or, on the public path, the share policy allowed it).
        let song = match (row.song_id, row.song_title) {
            (Some(id), Some(title)) => Some(SongSummary {
                id,
                title,
                artist: row.song_artist,
                default_key: row.song_default_key,
                tempo: row.song_tempo,
                time_signature: row.song_time_signature.unwrap_or_else(|| "4/4".to_string()),
                chordpro: row.song_chordpro.unwrap_or_default(),
            }),
            _ => None,
        };

        Self {
            id: row.id,
            setlist_id: row.setlist_id,
            position: row.position,
            kind: row.kind,
            song_id: row.song_id,
            title: row.title,
            notes: row.notes,
            key_override: row.key_override,
            capo_override: row.capo_override,
            duration_seconds: row.duration_seconds,
            song,
        }
    }
}

/// Always query `from setlist_items si left join songs s on s.id = si.song_id`.
pub const ITEM_COLUMNS: &str = "si.id, si.setlist_id, si.position, si.kind, si.song_id, \
     si.title, si.notes, si.key_override, si.capo_override, si.duration_seconds, \
     s.title as song_title, s.artist as song_artist, s.default_key as song_default_key, \
     s.tempo as song_tempo, s.time_signature as song_time_signature, s.chordpro as song_chordpro";

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub kind: String,
    pub song_id: Option<Uuid>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub key_override: Option<String>,
    pub capo_override: Option<i16>,
    pub duration_seconds: Option<i32>,
    pub position: Option<i32>,
}

#[derive(Debug, Default, Deserialize)]
pub struct UpdateItemRequest {
    #[serde(default, deserialize_with = "double_option")]
    pub kind: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub song_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "double_option")]
    pub title: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub notes: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub key_override: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub capo_override: Option<Option<i16>>,
    #[serde(default, deserialize_with = "double_option")]
    pub duration_seconds: Option<Option<i32>>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRequest {
    pub item_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct SetlistDetailResponse {
    pub setlist: Setlist,
    pub items: Vec<SetlistItem>,
}

#[derive(Debug, Serialize)]
pub struct ItemsResponse {
    pub items: Vec<SetlistItem>,
}

#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub share_token: String,
}
