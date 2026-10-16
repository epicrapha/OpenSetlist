//! M1 song library integration tests.
//!
//! Require a real Postgres via `DATABASE_URL`, same as `tests/security.rs`. Each
//! test builds its own pool/state (a `PgPool` is bound to the runtime that
//! created it; sharing one across `#[tokio::test]` runtimes hangs).

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
    content_type: &str,
    body: Vec<u8>,
    cookie: Option<&str>,
    workspace_id: Option<Uuid>,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", content_type);
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
        "application/json",
        body.to_string().into_bytes(),
        cookie,
        workspace_id,
    )
}

fn get_request(uri: &str, cookie: Option<&str>, workspace_id: Option<Uuid>) -> Request<Body> {
    request(
        "GET",
        uri,
        "application/json",
        Vec::new(),
        cookie,
        workspace_id,
    )
}

async fn send(router: &Router, http_request: Request<Body>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let response = router
        .clone()
        .oneshot(http_request)
        .await
        .expect("router call failed");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024 * 1024)
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

async fn register_and_login(router: &Router, tag: &str) -> String {
    let email = format!("{tag}-{}@example.test", Uuid::new_v4());
    let password = "password123";
    let (status, _, _) = send_json(
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
    session_cookie(&headers)
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
    assert_eq!(status, StatusCode::CREATED, "song creation failed");
    body
}

/// CRUD, duplicate and export round-trip (SONG-1..3, SONG-11).
#[tokio::test]
async fn song_crud_duplicate_and_export() {
    let (router, _state) = setup().await;
    let cookie = register_and_login(&router, "crud").await;
    let ws = create_workspace(&router, &cookie, "Band").await;

    let song = create_song(&router, &cookie, ws, "Amazing Grace").await;
    assert_eq!(song["title"], "Amazing Grace");
    assert_eq!(song["tags"], json!([]));
    assert_eq!(song["time_signature"], "4/4");
    let song_id = song["id"].as_str().unwrap().to_string();

    // Read.
    let (status, _, body) = send_json(
        &router,
        get_request(&format!("/api/v1/songs/{song_id}"), Some(&cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], song_id);

    // Partial update.
    let (status, _, body) = send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/songs/{song_id}"),
            json!({ "artist": "John Newton", "tempo": 90 }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["artist"], "John Newton");
    assert_eq!(body["tempo"], 90);

    // List with search.
    let (status, _, body) = send_json(
        &router,
        get_request("/api/v1/songs?q=amazing", Some(&cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);

    // Duplicate.
    let (status, _, dup) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/songs/{song_id}/duplicate"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_ne!(dup["id"], song_id);
    assert_eq!(dup["artist"], "John Newton");

    // Export plain text strips ChordPro markup.
    let (status, headers, bytes) = send(
        &router,
        get_request(
            &format!("/api/v1/songs/{song_id}/export?format=text"),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers["content-type"]
        .to_str()
        .unwrap()
        .starts_with("text/plain"));
    let text = String::from_utf8(bytes).unwrap();
    assert_eq!(text, "Amazing grace\n");

    // Export chordpro returns the stored source.
    let (status, _, bytes) = send(
        &router,
        get_request(
            &format!("/api/v1/songs/{song_id}/export"),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, b"[C]Amazing [G]grace");

    // Delete then read back 404.
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
    let (status, _, _) = send_json(
        &router,
        get_request(&format!("/api/v1/songs/{song_id}"), Some(&cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// RLS: a song in workspace A is invisible from workspace B, even by id.
#[tokio::test]
async fn songs_are_isolated_between_workspaces() {
    let (router, _state) = setup().await;
    let cookie_a = register_and_login(&router, "iso-a").await;
    let cookie_b = register_and_login(&router, "iso-b").await;
    let ws_a = create_workspace(&router, &cookie_a, "A").await;
    let ws_b = create_workspace(&router, &cookie_b, "B").await;

    let song = create_song(&router, &cookie_a, ws_a, "Secret Song").await;
    let song_id = song["id"].as_str().unwrap().to_string();

    // B cannot fetch A's song by id...
    let (status, _, _) = send_json(
        &router,
        get_request(
            &format!("/api/v1/songs/{song_id}"),
            Some(&cookie_b),
            Some(ws_b),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "cross-workspace read leaked");

    // ...nor list its assets.
    let (status, _, _) = send_json(
        &router,
        get_request(
            &format!("/api/v1/songs/{song_id}/assets"),
            Some(&cookie_b),
            Some(ws_b),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-workspace asset list leaked"
    );

    // ...nor list it.
    let (status, _, body) = send_json(
        &router,
        get_request("/api/v1/songs", Some(&cookie_b), Some(ws_b)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0);

    // And a non-member of ws_a cannot use it as a scope at all.
    let (status, _, _) = send_json(
        &router,
        get_request("/api/v1/songs", Some(&cookie_b), Some(ws_a)),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// Writes require leader+; reads only need membership.
#[tokio::test]
async fn writes_require_leader_role() {
    let (router, state) = setup().await;
    let owner_cookie = register_and_login(&router, "role-owner").await;
    let ws = create_workspace(&router, &owner_cookie, "Band").await;

    // Add a plain member directly (no accept-invite endpoint yet).
    let member_email = format!("member-{}@example.test", Uuid::new_v4());
    let (status, _, body) = send_json(
        &router,
        json_request(
            "POST",
            "/api/v1/auth/register",
            json!({ "email": member_email, "password": "password123", "name": "Member" }),
            None,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let member_id: Uuid = body["id"].as_str().unwrap().parse().unwrap();

    let mut tx = state.scoped(Some(ws), Some(member_id)).await.unwrap();
    sqlx::query("insert into memberships (user_id, workspace_id, role) values ($1, $2, 'member')")
        .bind(member_id)
        .bind(ws)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let (status, headers, _) = send_json(
        &router,
        json_request(
            "POST",
            "/api/v1/auth/login",
            json!({ "email": member_email, "password": "password123" }),
            None,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let member_cookie = session_cookie(&headers);

    // Member can read...
    let (status, _, _) = send_json(
        &router,
        get_request("/api/v1/songs", Some(&member_cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // ...but cannot write.
    let (status, _, body) = send_json(
        &router,
        json_request(
            "POST",
            "/api/v1/songs",
            json!({ "title": "Nope", "chordpro": "" }),
            Some(&member_cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "forbidden");

    // Missing workspace header is a validation error.
    let (status, _, body) = send_json(
        &router,
        json_request(
            "POST",
            "/api/v1/songs",
            json!({ "title": "Nope", "chordpro": "" }),
            Some(&owner_cookie),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");
}

fn multipart_pdf(content_type: &str, filename: &str, data: &[u8]) -> (String, Vec<u8>) {
    let boundary = "----opensetlistboundary";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

/// Link attach/delete plus PDF upload happy path and content validation.
#[tokio::test]
async fn links_and_pdf_assets() {
    let (router, _state) = setup().await;
    let cookie = register_and_login(&router, "assets").await;
    let ws = create_workspace(&router, &cookie, "Band").await;
    let song = create_song(&router, &cookie, ws, "Song").await;
    let song_id = song["id"].as_str().unwrap().to_string();

    // Reference link.
    let (status, _, link) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/songs/{song_id}/assets"),
            json!({ "kind": "link", "url": "https://example.test/chart", "label": "Chart" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(link["kind"], "link");
    assert_eq!(link["url"], "https://example.test/chart");

    // A non-http(s) scheme is rejected.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/songs/{song_id}/assets"),
            json!({ "kind": "link", "url": "javascript:alert(1)" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Valid PDF upload.
    let (content_type, body) = multipart_pdf("application/pdf", "chart.pdf", b"%PDF-1.4\n%%EOF\n");
    let (status, _, asset) = send_json(
        &router,
        request(
            "POST",
            &format!("/api/v1/songs/{song_id}/assets/pdf"),
            &content_type,
            body,
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "valid pdf upload failed");
    assert_eq!(asset["kind"], "pdf");
    assert_eq!(asset["filename"], "chart.pdf");
    assert_eq!(asset["content_type"], "application/pdf");
    assert_eq!(
        asset["object_name"],
        Value::Null,
        "object name must not leak"
    );
    let asset_id = asset["id"].as_str().unwrap().to_string();

    // Download round-trips the bytes.
    let (status, headers, bytes) = send(
        &router,
        get_request(
            &format!("/api/v1/assets/{asset_id}/file"),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers["content-type"], "application/pdf");
    assert!(bytes.starts_with(b"%PDF-"));

    // Listing returns both the link and the PDF, oldest first, with the
    // contract fields and no internal object name.
    let (status, _, listed) = send_json(
        &router,
        get_request(
            &format!("/api/v1/songs/{song_id}/assets"),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let assets = listed["assets"].as_array().unwrap();
    assert_eq!(assets.len(), 2, "expected link + pdf");
    assert_eq!(assets[0]["id"], link["id"]);
    assert_eq!(assets[0]["kind"], "link");
    assert_eq!(assets[0]["url"], "https://example.test/chart");
    assert_eq!(assets[1]["id"], asset_id);
    assert_eq!(assets[1]["kind"], "pdf");
    assert_eq!(assets[1]["filename"], "chart.pdf");
    assert_eq!(assets[1]["content_type"], "application/pdf");
    assert_eq!(
        assets
            .iter()
            .filter(|a| a["object_name"] != Value::Null)
            .count(),
        0
    );

    // Content-Type says PDF but the magic bytes do not.
    let (content_type, body) = multipart_pdf("application/pdf", "fake.pdf", b"not a pdf");
    let (status, _, _) = send_json(
        &router,
        request(
            "POST",
            &format!("/api/v1/songs/{song_id}/assets/pdf"),
            &content_type,
            body,
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

    // Oversized file (> 10 MB) is rejected with 413.
    let mut big = Vec::with_capacity(10 * 1024 * 1024 + 16);
    big.extend_from_slice(b"%PDF-");
    big.resize(10 * 1024 * 1024 + 1, b'A');
    let (content_type, body) = multipart_pdf("application/pdf", "big.pdf", &big);
    let (status, _, _) = send_json(
        &router,
        request(
            "POST",
            &format!("/api/v1/songs/{song_id}/assets/pdf"),
            &content_type,
            body,
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);

    // Delete the link asset.
    let (status, _, _) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/assets/{}", link["id"].as_str().unwrap()),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

/// Arrangement create/list/update/delete happy path.
#[tokio::test]
async fn arrangements_lifecycle() {
    let (router, _state) = setup().await;
    let cookie = register_and_login(&router, "arr").await;
    let ws = create_workspace(&router, &cookie, "Band").await;
    let song = create_song(&router, &cookie, ws, "Song").await;
    let song_id = song["id"].as_str().unwrap().to_string();

    let (status, _, arr) = send_json(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/songs/{song_id}/arrangements"),
            json!({ "name": "Acoustic", "key": "G", "chordpro": "[G]la" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(arr["name"], "Acoustic");
    assert_eq!(arr["song_id"], song_id);
    let arr_id = arr["id"].as_str().unwrap().to_string();

    let (status, _, list) = send_json(
        &router,
        get_request(
            &format!("/api/v1/songs/{song_id}/arrangements"),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    let (status, _, updated) = send_json(
        &router,
        json_request(
            "PATCH",
            &format!("/api/v1/arrangements/{arr_id}"),
            json!({ "key": "A" }),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["key"], "A");
    assert_eq!(updated["name"], "Acoustic");

    let (status, _, _) = send_json(
        &router,
        json_request(
            "DELETE",
            &format!("/api/v1/arrangements/{arr_id}"),
            json!({}),
            Some(&cookie),
            Some(ws),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

/// Non-members get 404 on a workspace they do not belong to; no auth is 401.
#[tokio::test]
async fn songs_require_authentication() {
    let (router, _state) = setup().await;
    let ws = Uuid::new_v4();
    let (status, _, body) = send_json(&router, get_request("/api/v1/songs", None, Some(ws))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthorized");
}
