//! Invite flow integration tests (PLAN.md §3.2).
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

fn json_request(method: &str, uri: &str, body: Value, cookie: Option<&str>) -> Request<Body> {
    request(method, uri, body.to_string().into_bytes(), cookie, None)
}

fn get_request(uri: &str, cookie: Option<&str>, workspace_id: Option<Uuid>) -> Request<Body> {
    request("GET", uri, Vec::new(), cookie, workspace_id)
}

async fn send(router: &Router, http_request: Request<Body>) -> (StatusCode, HeaderMap, Value) {
    let response = router
        .clone()
        .oneshot(http_request)
        .await
        .expect("router call failed");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("failed to read body");
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

    let (status, _, body) = send(
        router,
        json_request(
            "POST",
            "/api/v1/auth/register",
            json!({ "email": email, "password": password, "name": tag }),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "register failed");
    let user_id: Uuid = body["id"].as_str().unwrap().parse().unwrap();

    let (status, headers, _) = send(
        router,
        json_request(
            "POST",
            "/api/v1/auth/login",
            json!({ "email": email, "password": password }),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "login failed");
    (session_cookie(&headers), email, user_id)
}

async fn create_workspace(router: &Router, cookie: &str, name: &str) -> Uuid {
    let (status, _, body) = send(
        router,
        json_request(
            "POST",
            "/api/v1/workspaces",
            json!({ "name": name }),
            Some(cookie),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "workspace creation failed");
    body["id"].as_str().unwrap().parse().unwrap()
}

async fn create_invite(router: &Router, cookie: &str, ws: Uuid, email: &str, role: &str) -> Value {
    let (status, _, body) = send(
        router,
        json_request(
            "POST",
            &format!("/api/v1/workspaces/{ws}/invites"),
            json!({ "email": email, "role": role }),
            Some(cookie),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "invite creation failed: {body}"
    );
    body
}

async fn accept(router: &Router, cookie: &str, token: &str) -> (StatusCode, Value) {
    let (status, _, body) = send(
        router,
        json_request(
            "POST",
            &format!("/api/v1/invites/{token}/accept"),
            json!({}),
            Some(cookie),
        ),
    )
    .await;
    (status, body)
}

/// Happy path: accept joins the caller with the invited role, the Invite carries
/// the contract fields, and the new member can use the workspace scope.
#[tokio::test]
async fn accept_invite_joins_workspace() {
    let (router, _state) = setup().await;
    let (owner_cookie, _owner_email, _) = register_and_login(&router, "owner").await;
    let ws = create_workspace(&router, &owner_cookie, "Band").await;

    let (invitee_cookie, invitee_email, _) = register_and_login(&router, "invitee").await;
    let invite = create_invite(&router, &owner_cookie, ws, &invitee_email, "leader").await;

    // Frozen Invite shape.
    assert!(invite["id"].is_string());
    assert_eq!(invite["workspace_id"], ws.to_string());
    assert_eq!(invite["email"], invitee_email);
    assert_eq!(invite["role"], "leader");
    assert!(invite["token"].as_str().unwrap().len() >= 20);
    assert!(invite["expires_at"].is_string());
    assert_eq!(invite["accepted_at"], Value::Null);
    assert!(invite["created_at"].is_string());
    let token = invite["token"].as_str().unwrap();

    let (status, body) = accept(&router, &invitee_cookie, token).await;
    assert_eq!(status, StatusCode::CREATED, "accept failed: {body}");
    assert_eq!(body["workspace_id"], ws.to_string());
    assert_eq!(body["role"], "leader");

    // The caller is now a member with the invited role.
    let (status, _, me) = send(
        &router,
        get_request("/api/v1/auth/me", Some(&invitee_cookie), None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let membership = me["memberships"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["workspace_id"] == ws.to_string())
        .expect("new member should appear in /auth/me");
    assert_eq!(membership["role"], "leader");

    // And can now use the workspace scope (viewer+ can read songs).
    let (status, _, _) = send(
        &router,
        get_request("/api/v1/songs", Some(&invitee_cookie), Some(ws)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // A second accept is rejected.
    let (status, _) = accept(&router, &invitee_cookie, token).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

/// Unknown token is a 404.
#[tokio::test]
async fn accept_unknown_token_is_404() {
    let (router, _state) = setup().await;
    let (cookie, _, _) = register_and_login(&router, "nobody").await;

    let (status, body) = accept(&router, &cookie, "does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND, "got {body}");
    assert_eq!(body["error"]["code"], "not_found");
}

/// An invite is email-bound: a different account gets 403.
#[tokio::test]
async fn accept_wrong_email_is_403() {
    let (router, _state) = setup().await;
    let (owner_cookie, _, _) = register_and_login(&router, "owner").await;
    let ws = create_workspace(&router, &owner_cookie, "Band").await;

    let (_invitee_cookie, invitee_email, _) = register_and_login(&router, "invitee").await;
    let (intruder_cookie, _, _) = register_and_login(&router, "intruder").await;
    let invite = create_invite(&router, &owner_cookie, ws, &invitee_email, "member").await;
    let token = invite["token"].as_str().unwrap();

    let (status, body) = accept(&router, &intruder_cookie, token).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "got {body}");
    assert_eq!(body["error"]["code"], "forbidden");
}

/// An expired invite is rejected with 409.
#[tokio::test]
async fn accept_expired_invite_is_409() {
    let (router, state) = setup().await;
    let (owner_cookie, _owner_email, owner_id) = register_and_login(&router, "owner").await;
    let ws = create_workspace(&router, &owner_cookie, "Band").await;

    let (invitee_cookie, invitee_email, _) = register_and_login(&router, "invitee").await;
    let invite = create_invite(&router, &owner_cookie, ws, &invitee_email, "member").await;
    let token = invite["token"].as_str().unwrap().to_string();

    // Force expiry directly.
    let mut tx = state.scoped(Some(ws), Some(owner_id)).await.unwrap();
    sqlx::query("update invites set expires_at = now() - interval '1 day' where token = $1")
        .bind(&token)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let (status, body) = accept(&router, &invitee_cookie, &token).await;
    assert_eq!(status, StatusCode::CONFLICT, "got {body}");
}

/// `role: "owner"` is rejected when creating an invite.
#[tokio::test]
async fn create_invite_rejects_owner_role() {
    let (router, _state) = setup().await;
    let (owner_cookie, _, _) = register_and_login(&router, "owner").await;
    let ws = create_workspace(&router, &owner_cookie, "Band").await;

    let (status, _, body) = send(
        &router,
        json_request(
            "POST",
            &format!("/api/v1/workspaces/{ws}/invites"),
            json!({ "email": "someone@example.test", "role": "owner" }),
            Some(&owner_cookie),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");
}

/// GET invites: 200 for owner, 403 for a non-admin member, 404 for a non-member.
#[tokio::test]
async fn list_invites_is_admin_only() {
    let (router, state) = setup().await;
    let (owner_cookie, _, _) = register_and_login(&router, "owner").await;
    let ws = create_workspace(&router, &owner_cookie, "Band").await;

    let invitee_email = format!("invitee-{}@example.test", Uuid::new_v4());
    create_invite(&router, &owner_cookie, ws, &invitee_email, "member").await;

    // Plain member added directly (no accept endpoint needed).
    let (member_cookie, member_email, member_id) = register_and_login(&router, "member").await;
    let mut tx = state.scoped(Some(ws), Some(member_id)).await.unwrap();
    sqlx::query("insert into memberships (user_id, workspace_id, role) values ($1, $2, 'member')")
        .bind(member_id)
        .bind(ws)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let _ = member_email;

    // Owner: 200 with the pending invite in the frozen shape.
    let (status, _, body) = send(
        &router,
        get_request(
            &format!("/api/v1/workspaces/{ws}/invites"),
            Some(&owner_cookie),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let invites = body["invites"].as_array().unwrap();
    assert_eq!(invites.len(), 1);
    assert!(invites[0]["token"].is_string());
    assert!(invites[0]["created_at"].is_string());
    assert_eq!(invites[0]["accepted_at"], Value::Null);

    // Non-admin member: 403.
    let (status, _, _) = send(
        &router,
        get_request(
            &format!("/api/v1/workspaces/{ws}/invites"),
            Some(&member_cookie),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Non-member: 404 (no leak that the workspace exists).
    let (stranger_cookie, _, _) = register_and_login(&router, "stranger").await;
    let (status, _, _) = send(
        &router,
        get_request(
            &format!("/api/v1/workspaces/{ws}/invites"),
            Some(&stranger_cookie),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Unauthenticated: 401.
    let (status, _, _) = send(
        &router,
        get_request(&format!("/api/v1/workspaces/{ws}/invites"), None, None),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Unauthenticated accept is 401.
#[tokio::test]
async fn accept_requires_session() {
    let (router, _state) = setup().await;
    let (status, _, body) = send(
        &router,
        json_request("POST", "/api/v1/invites/any-token/accept", json!({}), None),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthorized");
}
