//! M2 setlist integration tests (PLAN.md §3.3).
//!
//! Require a real Postgres via `DATABASE_URL`. Each test builds its own
//! pool/state (a `PgPool` is bound to the runtime that created it).

use std::sync::Arc;

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    Router,
};
use opensetlist_server::{build_router, build_state, config::Config, state::AppState};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

async fn setup() -> (Router, Arc<AppState>) {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the OpenSetlist integration tests");
    let config = Config {
        database_url,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        cookie_secure: false,
        session_secret: "test-session-secret-at-least-32-bytes!".to_string(),
        upload_dir: std::env::temp_dir().join(format!("opensetlist-test-{}", Uuid::new_v4())),
    };
    let state = build_state(config)
        .await
        .expect("failed to build app state");
    let router = build_router(state.clone());
    (router, state)
}

fn request(
    method: &str,
    uri: &str,
    body: Vec<u8>,
    cookie: Option<&str>,
    workspace_id: Option<Uuid>,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    if let Some(workspace_id) = workspace_id {
        builder = builder.header("x-workspace-id", workspace_id.to_string());
    }
    builder
        .body(Body::from(body))
        .expect("failed to build request")
}

fn json_request(
    method: &str,
    uri: &str,
    body: Value,
    cookie: Option<&str>,
    workspace_id: Option<Uuid>,
) -> Request<Body> {
    request(
        method,
        uri,
        body.to_string().into_bytes(),
        cookie,
        workspace_id,
    )
}

fn get_request(uri: &str, cookie: Option<&str>, workspace_id: Option<Uuid>) -> Request<Body> {
    request("GET", uri, Vec::new(), cookie, workspace_id)
}

async fn send(router: &Router, http_request: Request<Body>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let response = router
        .clone()
        .oneshot(http_request)
        .await
        .expect("router call failed");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 16 * 1024 * 1024)
        .await
        .expect("failed to read body");
    (status, headers, bytes.to_vec())
}

async fn send_json(router: &Router, http_request: Request<Body>) -> (StatusCode, HeaderMap, Value) {
    let (status, headers, bytes) = send(router, http_request).await;
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, value)
}

fn session_cookie(headers: &HeaderMap) -> String {
    headers
        .get("set-cookie")
        .expect("expected a Set-Cookie header")
        .to_str()
        .expect("cookie header not utf-8")
        .split(';')
        .next()
        .expect("empty cookie")
        .to_string()
}

/// Register a user and return `(session_cookie, email, user_id)`.
async fn register_and_login(router: &Router, tag: &str) -> (String, String, Uuid) {
    let email = format!("{tag}-{}@example.test", Uuid::new_v4());
    let password = "password123";

    let (status, _, body) = send_json(
        router,
        json_request(
            "POST",
            "/api/v1/auth/register",
            json!({ "email": email, "password": password, "name": tag }),
            None,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "register failed");
    let user_id: Uuid = body["id"].as_str().unwrap().parse().unwrap();

    let (status, headers, _) = send_json(
        router,
        json_request(
            "POST",
            "/api/v1/auth/login",
            json!({ "email": email, "password": password }),
            None,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "login failed");
    (session_cookie(&headers), email, user_id)
}

async fn create_workspace(router: &Router, cookie: &str, name: &str) -> Uuid {
    let (status, _, body) = send_json(
        router,
        json_request(
            "POST",
            "/api/v1/workspaces",
            json!({ "name": name }),
            Some(cookie),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "workspace creation failed");
    body["id"].as_str().unwrap().parse().unwrap()
}

async fn create_song(router: &Router, cookie: &str, ws: Uuid, title: &str) -> Value {
    let (status, _, body) = send_json(
        router,
        json_request(
            "POST",
            "/api/v1/songs",
            json!({ "title": title, "chordpro": "[C]Amazing [G]grace" }),
            Some(cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "song creation failed: {body}");
    body
}

async fn create_setlist(router: &Router, cookie: &str, ws: Uuid, body: Value) -> Value {
    let (status, _, value) = send_json(
        router,
        json_request("POST", "/api/v1/setlists", body, Some(cookie), Some(ws)),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "setlist creation failed: {value}"
    );
    value
}

async fn add_member(state: &AppState, ws: Uuid, user_id: Uuid, role: &str) {
    let mut tx = state.scoped(Some(ws), Some(user_id)).await.unwrap();
    sqlx::query("insert into memberships (user_id, workspace_id, role) values ($1, $2, $3::membership_role)")
        .bind(user_id)
        .bind(ws)
        .bind(role)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

async fn detail(router: &Router, cookie: &str, ws: Uuid, id: &str) -> Value {
    let (status, _, body) = send_json(
        router,
        get_request(&format!("/api/v1/setlists/{id}"), Some(cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get setlist failed: {body}");
    body
}

/// create → list → get → patch → delete lifecycle.
#[tokio::test]
async fn setlist_lifecycle() {
    let (router, _state) = setup().await;
    let (cookie, _, _) = register_and_login(&router, "lifecycle").await;
    let ws = create_workspace(&router, &cookie, "Band").await;

    let setlist = create_setlist(
        &router,
        &cookie,
        ws,
        json!({ "title": "Sunday", "service_date": "2026-01-04", "service_type": "service" }),
    )
    .await;
    assert_eq!(setlist["title"], "Sunday");
    assert_eq!(setlist["service_date"], "2026-01-04");
    assert_eq!(setlist["service_type"], "service");
    assert_eq!(setlist["status"], "draft");
    assert_eq!(setlist["is_template"], false);
    assert_eq!(setlist["share_token"], Value::Null);
    assert_eq!(setlist["item_count"], 0);
    let id = setlist["id"].as_str().unwrap().to_string();

    // List.
    let (status, _, body) = send_json(
        &router,
        get_request("/api/v1/setlists", Some(&cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["setlists"][0]["id"], id);

    // Get returns the header + empty items.
    let got = detail(&router, &cookie, ws, &id).await;
    assert_eq!(got["setlist"]["id"], id);
    assert_eq!(got["items"], json!([]));

    // Patch including clearing an optional field.
    let (status, _, patched) = send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/setlists/{id}"),
            json!({ "title": "Sunday AM", "status": "ready", "service_type": null }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patched["title"], "Sunday AM");
    assert_eq!(patched["status"], "ready");
    assert_eq!(patched["service_type"], Value::Null);

    // Invalid status is rejected.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/setlists/{id}"),
            json!({ "status": "published" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Delete.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/setlists/{id}"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = send_json(
        &router,
        get_request(&format!("/api/v1/setlists/{id}"), Some(&cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// `song` and `note` items, item validation, and overrides never mutating songs.
#[tokio::test]
async fn items_and_overrides() {
    let (router, _state) = setup().await;
    let (cookie, _, _) = register_and_login(&router, "items").await;
    let ws = create_workspace(&router, &cookie, "Band").await;
    let song = create_song(&router, &cookie, ws, "Amazing Grace").await;
    let song_id = song["id"].as_str().unwrap().to_string();
    let setlist = create_setlist(&router, &cookie, ws, json!({ "title": "Sunday" })).await;
    let setlist_id = setlist["id"].as_str().unwrap().to_string();

    // Song item.
    let (status, _, item) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({
                "kind": "song",
                "song_id": song_id,
                "key_override": "G",
                "capo_override": 2,
                "notes": "intro"
            }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "song item failed: {item}");
    assert_eq!(item["position"], 0);
    assert_eq!(item["kind"], "song");
    assert_eq!(item["key_override"], "G");
    assert_eq!(item["capo_override"], 2);
    assert_eq!(item["notes"], "intro");
    assert_eq!(item["song"]["id"], song_id);
    assert_eq!(item["song"]["title"], "Amazing Grace");
    let item_id = item["id"].as_str().unwrap().to_string();

    // Note item.
    let (status, _, note) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({ "kind": "note", "title": "Welcome", "duration_seconds": 60 }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(note["position"], 1);
    assert_eq!(note["song"], Value::Null);

    // Note without a title → 400.
    let (status, _, body) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({ "kind": "note" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "bad_request");

    // Song item with a song from another workspace → 404.
    let (other_cookie, _, _) = register_and_login(&router, "other").await;
    let other_ws = create_workspace(&router, &other_cookie, "Other").await;
    let other_song = create_song(&router, &other_cookie, other_ws, "Elsewhere").await;
    let (status, _, _) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({ "kind": "song", "song_id": other_song["id"] }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Patching item overrides must not touch the song.
    let (status, _, item) = send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/setlists/{setlist_id}/items/{item_id}"),
            json!({ "key_override": "A", "capo_override": 4, "notes": null }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(item["key_override"], "A");
    assert_eq!(item["capo_override"], 4);
    assert_eq!(item["notes"], Value::Null);

    let (status, _, song_after) = send_json(
        &router,
        get_request(&format!("/api/v1/songs/{song_id}"), Some(&cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(song_after["default_key"], Value::Null);
    assert_eq!(song_after["tempo"], Value::Null);
    assert_eq!(song_after["chordpro"], song["chordpro"]);
}

/// Reorder persists, positions stay dense, wrong id sets are rejected.
#[tokio::test]
async fn reorder_is_dense_and_validated() {
    let (router, _state) = setup().await;
    let (cookie, _, _) = register_and_login(&router, "reorder").await;
    let ws = create_workspace(&router, &cookie, "Band").await;
    let setlist = create_setlist(&router, &cookie, ws, json!({ "title": "Sunday" })).await;
    let setlist_id = setlist["id"].as_str().unwrap().to_string();

    let mut ids = Vec::new();
    for title in ["A", "B", "C"] {
        let (status, _, item) = send_json(
            &router,
            json_request(
                "POST",
                &format!("/api/v1/setlists/{setlist_id}/items"),
                json!({ "kind": "note", "title": title }),
                Some(&cookie),
                Some(ws),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        ids.push(item["id"].as_str().unwrap().to_string());
    }

    let reversed: Vec<&String> = ids.iter().rev().collect();
    let (status, _, body) = send_json(
        &router,
        json_request(
            "PUT",
            &format!("/api/v1/setlists/{setlist_id}/items/order"),
            json!({ "item_ids": reversed }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "reorder failed: {body}");
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    for (position, item) in items.iter().enumerate() {
        assert_eq!(item["position"], position as i64);
    }
    assert_eq!(items[0]["id"], ids[2]);
    assert_eq!(items[1]["id"], ids[1]);
    assert_eq!(items[2]["id"], ids[0]);

    // Persisted.
    let got = detail(&router, &cookie, ws, &setlist_id).await;
    assert_eq!(got["items"][0]["id"], ids[2]);

    // Wrong id set (missing one) is rejected.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "PUT",
            &format!("/api/v1/setlists/{setlist_id}/items/order"),
            json!({ "item_ids": [ids[0], ids[1]] }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Deleting the middle item keeps positions dense.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/setlists/{setlist_id}/items/{}", ids[1]),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let got = detail(&router, &cookie, ws, &setlist_id).await;
    let items = got["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["position"], 0);
    assert_eq!(items[1]["position"], 1);
}

/// A setlist in another workspace is invisible (404), even by id.
#[tokio::test]
async fn cross_tenant_is_404() {
    let (router, _state) = setup().await;
    let (cookie_a, _, _) = register_and_login(&router, "tenant-a").await;
    let ws_a = create_workspace(&router, &cookie_a, "A").await;
    let (cookie_b, _, _) = register_and_login(&router, "tenant-b").await;
    let ws_b = create_workspace(&router, &cookie_b, "B").await;

    let setlist = create_setlist(&router, &cookie_a, ws_a, json!({ "title": "Secret" })).await;
    let id = setlist["id"].as_str().unwrap();

    let (status, _, _) = send_json(
        &router,
        get_request(
            &format!("/api/v1/setlists/{id}"),
            Some(&cookie_b),
            Some(ws_b),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _, body) = send_json(
        &router,
        get_request("/api/v1/setlists", Some(&cookie_b), Some(ws_b)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0);

    // Even a patch (which passes the leader gate) cannot reach it.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/setlists/{id}"),
            json!({ "title": "hijacked" }),
            Some(&cookie_b),
            Some(ws_b),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// Viewer can read; viewer cannot write.
#[tokio::test]
async fn viewer_role_gates() {
    let (router, state) = setup().await;
    let (owner_cookie, _, _) = register_and_login(&router, "owner").await;
    let ws = create_workspace(&router, &owner_cookie, "Band").await;
    create_setlist(&router, &owner_cookie, ws, json!({ "title": "Sunday" })).await;

    let (viewer_cookie, _, viewer_id) = register_and_login(&router, "viewer").await;
    add_member(&state, ws, viewer_id, "viewer").await;

    let (status, _, body) = send_json(
        &router,
        get_request("/api/v1/setlists", Some(&viewer_cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "viewer read failed: {body}");

    let (status, _, body) = send_json(
        &router,
        json_request(
            "POST",
            "/api/v1/setlists",
            json!({ "title": "Nope" }),
            Some(&viewer_cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "forbidden");
}

/// Archived setlists block item mutations and share creation, but reads and an
/// already-created public share keep working.
#[tokio::test]
async fn archived_blocks_writes() {
    let (router, _state) = setup().await;
    let (cookie, _, _) = register_and_login(&router, "archived").await;
    let ws = create_workspace(&router, &cookie, "Band").await;
    let setlist = create_setlist(&router, &cookie, ws, json!({ "title": "Sunday" })).await;
    let setlist_id = setlist["id"].as_str().unwrap().to_string();

    let (status, _, item) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({ "kind": "note", "title": "Welcome" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let item_id = item["id"].as_str().unwrap().to_string();

    // Share before archiving so we can prove public reads still work.
    let (status, _, share) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/share"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = share["share_token"].as_str().unwrap().to_string();

    let (status, _, _) = send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/setlists/{setlist_id}"),
            json!({ "status": "archived" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Item create / patch / delete / reorder and share creation → 409.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({ "kind": "note", "title": "Late" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, _, _) = send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/setlists/{setlist_id}/items/{item_id}"),
            json!({ "notes": "nope" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, _, _) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/setlists/{setlist_id}/items/{item_id}"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, _, _) = send_json(
        &router,
        json_request(
            "PUT",
            &format!("/api/v1/setlists/{setlist_id}/items/order"),
            json!({ "item_ids": [item_id] }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Reads still work.
    let (status, _, _) = send_json(
        &router,
        get_request(
            &format!("/api/v1/setlists/{setlist_id}"),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Public share still works.
    let (status, _, _) = send_json(
        &router,
        get_request(&format!("/api/v1/shared/setlists/{token}"), None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

/// Duplicate copies items and overrides, resetting status/template/share token.
#[tokio::test]
async fn duplicate_resets_and_copies() {
    let (router, _state) = setup().await;
    let (cookie, _, _) = register_and_login(&router, "dup").await;
    let ws = create_workspace(&router, &cookie, "Band").await;
    let song = create_song(&router, &cookie, ws, "Amazing Grace").await;

    let setlist = create_setlist(
        &router,
        &cookie,
        ws,
        json!({ "title": "Sunday", "service_date": "2026-01-04" }),
    )
    .await;
    let setlist_id = setlist["id"].as_str().unwrap().to_string();

    send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({ "kind": "song", "song_id": song["id"], "key_override": "G", "capo_override": 2 }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;

    // Make the source ready + template + shared, so resets are observable.
    send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/setlists/{setlist_id}"),
            json!({ "status": "ready", "is_template": true }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/share"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;

    let (status, _, dup) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/duplicate"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "duplicate failed: {dup}");
    assert_ne!(dup["id"], setlist["id"]);
    assert_eq!(dup["status"], "draft");
    assert_eq!(dup["is_template"], false);
    assert_eq!(dup["share_token"], Value::Null);
    assert_eq!(dup["item_count"], 1);
    assert_eq!(dup["service_date"], "2026-01-04");

    let dup_detail = detail(&router, &cookie, ws, dup["id"].as_str().unwrap()).await;
    let items = dup_detail["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["song_id"], song["id"]);
    assert_eq!(items[0]["key_override"], "G");
    assert_eq!(items[0]["capo_override"], 2);
}

/// Share creation is idempotent; public GET needs no session/header; revoke 404s.
#[tokio::test]
async fn public_share_and_revoke() {
    let (router, _state) = setup().await;
    let (cookie, email, _) = register_and_login(&router, "share").await;
    let ws = create_workspace(&router, &cookie, "Band").await;
    let song = create_song(&router, &cookie, ws, "Amazing Grace").await;
    let setlist = create_setlist(&router, &cookie, ws, json!({ "title": "Sunday" })).await;
    let setlist_id = setlist["id"].as_str().unwrap().to_string();

    send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({ "kind": "song", "song_id": song["id"], "key_override": "G" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;

    let (status, _, first) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/share"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = first["share_token"].as_str().unwrap().to_string();
    assert!(token.len() >= 32, "share token too short: {token}");
    assert!(token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));

    // Idempotent: same token.
    let (status, _, second) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/share"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(second["share_token"], token);

    // Public: no cookie, no X-Workspace-Id.
    let (status, _, public) = send_json(
        &router,
        get_request(&format!("/api/v1/shared/setlists/{token}"), None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "public share failed: {public}");
    assert_eq!(public["setlist"]["id"], setlist_id);
    let items = public["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["song"]["id"], song["id"]);
    assert_eq!(items[0]["song"]["title"], "Amazing Grace");
    assert_eq!(items[0]["key_override"], "G");
    // No account details leak.
    assert!(!public.to_string().contains(email.as_str()));

    // Revoke → old token 404s.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/setlists/{setlist_id}/share"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = send_json(
        &router,
        get_request(&format!("/api/v1/shared/setlists/{token}"), None, None),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// A public share exposes only its own workspace: not another workspace's
/// setlist or songs.
#[tokio::test]
async fn share_never_leaks_other_workspaces() {
    let (router, _state) = setup().await;
    let (cookie_a, _, _) = register_and_login(&router, "share-a").await;
    let ws_a = create_workspace(&router, &cookie_a, "A").await;
    let song_a = create_song(&router, &cookie_a, ws_a, "Song A").await;
    let setlist_a = create_setlist(&router, &cookie_a, ws_a, json!({ "title": "Set A" })).await;
    let setlist_a_id = setlist_a["id"].as_str().unwrap().to_string();
    send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_a_id}/items"),
            json!({ "kind": "song", "song_id": song_a["id"] }),
            Some(&cookie_a),
            Some(ws_a),
        ),
    )
    .await;

    let (cookie_b, _, _) = register_and_login(&router, "share-b").await;
    let ws_b = create_workspace(&router, &cookie_b, "B").await;
    let song_b = create_song(&router, &cookie_b, ws_b, "Song B").await;
    let setlist_b = create_setlist(&router, &cookie_b, ws_b, json!({ "title": "Set B" })).await;

    let (_, _, share) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_a_id}/share"),
            json!({}),
            Some(&cookie_a),
            Some(ws_a),
        ),
    )
    .await;
    let token = share["share_token"].as_str().unwrap();

    let (status, _, public) = send_json(
        &router,
        get_request(&format!("/api/v1/shared/setlists/{token}"), None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let text = public.to_string();
    assert_eq!(public["setlist"]["id"], setlist_a_id);
    assert_eq!(public["setlist"]["workspace_id"], ws_a.to_string());
    assert!(!text.contains(setlist_b["id"].as_str().unwrap()));
    assert!(!text.contains(song_b["id"].as_str().unwrap()));
    assert!(!text.contains("Song B"));
    assert!(!text.contains("Set B"));
    assert!(text.contains(song_a["id"].as_str().unwrap()));

    // B's setlist has no share token, so it cannot be reached anonymously.
    let (status, _, _) = send_json(
        &router,
        get_request("/api/v1/shared/setlists/not-a-token", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// Deleting a song referenced by a setlist is a 409; after removing the item it
/// succeeds.
#[tokio::test]
async fn deleting_referenced_song_is_409() {
    let (router, _state) = setup().await;
    let (cookie, _, _) = register_and_login(&router, "song-delete").await;
    let ws = create_workspace(&router, &cookie, "Band").await;
    let song = create_song(&router, &cookie, ws, "Amazing Grace").await;
    let song_id = song["id"].as_str().unwrap().to_string();
    let setlist = create_setlist(&router, &cookie, ws, json!({ "title": "Sunday" })).await;
    let setlist_id = setlist["id"].as_str().unwrap().to_string();

    let (status, _, item) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/setlists/{setlist_id}/items"),
            json!({ "kind": "song", "song_id": song_id }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let item_id = item["id"].as_str().unwrap().to_string();

    let (status, _, body) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/songs/{song_id}"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "song delete should be blocked"
    );
    assert_eq!(body["error"]["code"], "conflict");

    // Remove the reference, then the delete succeeds.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/setlists/{setlist_id}/items/{item_id}"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/songs/{song_id}"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}
