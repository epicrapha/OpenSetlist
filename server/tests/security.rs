//! Executable security tests.
//!
//! These require a real Postgres reachable via `DATABASE_URL`. They prove three
//! things behaviourally (not by inspection):
//!
//! 1. RLS blocks cross-tenant reads.
//! 2. A non-owner/non-admin cannot create invites.
//! 3. Unauthenticated requests to protected routes return 401.

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

/// Build a fresh app state per test.
///
/// A `PgPool` is bound to the Tokio runtime that created it, and every
/// `#[tokio::test]` gets its own runtime. A pool shared across tests (e.g. via a
/// static `OnceCell`) has its worker tasks dropped when the creating runtime
/// shuts down, leaking pool permits and hanging the next test with
/// `PoolTimedOut`. Building per test keeps the pool inside the runtime using it.
async fn setup() -> (Router, Arc<AppState>) {
    let database_url = std::env::var("DATABASE_URL").expect(
        "DATABASE_URL must be set to run the OpenSetlist integration tests \
         (e.g. postgres://opensetlist:change-me@localhost:5432/opensetlist)",
    );
    let config = Config {
        database_url,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        cookie_secure: false,
        upload_dir: std::path::PathBuf::from("./uploads"),
    };
    let state = build_state(config)
        .await
        .expect("failed to build app state");
    let router = build_router(state.clone());
    (router, state)
}

fn json_request(method: &str, uri: &str, body: Value, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    builder
        .body(Body::from(body.to_string()))
        .expect("failed to build request")
}

fn get_request(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    builder
        .body(Body::empty())
        .expect("failed to build request")
}

async fn send(router: &Router, request: Request<Body>) -> (StatusCode, HeaderMap, Value) {
    let response = router
        .clone()
        .oneshot(request)
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

async fn register_and_login(router: &Router, tag: &str) -> String {
    let email = format!("{tag}-{}@example.test", Uuid::new_v4());
    let password = "password123";

    let (status, _, _) = send(
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
    session_cookie(&headers)
}

/// (c) Unauthenticated access to protected routes returns 401.
#[tokio::test]
async fn unauthenticated_requests_are_unauthorized() {
    let (router, _state) = setup().await;

    let (status, _, body) = send(
        &router,
        json_request("POST", "/api/v1/workspaces", json!({ "name": "x" }), None),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthorized");

    let (status, _, body) = send(&router, get_request("/api/v1/auth/me", None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthorized");
}

/// (b) A member (not owner/admin) cannot create invites; the owner can.
#[tokio::test]
async fn non_admin_cannot_create_invites() {
    let (router, state) = setup().await;

    let owner_cookie = register_and_login(&router, "owner").await;

    let (status, _, body) = send(
        &router,
        json_request(
            "POST",
            "/api/v1/workspaces",
            json!({ "name": "Band" }),
            Some(&owner_cookie),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "workspace creation failed");
    let workspace_id: Uuid = body["id"].as_str().unwrap().parse().unwrap();

    // Register a second user and add them directly as a plain member (M0 has no
    // accept-invite endpoint yet).
    let member_email = format!("member-{}@example.test", Uuid::new_v4());
    let (status, _, body) = send(
        &router,
        json_request(
            "POST",
            "/api/v1/auth/register",
            json!({ "email": member_email, "password": "password123", "name": "Member" }),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let member_id: Uuid = body["id"].as_str().unwrap().parse().unwrap();

    let mut tx = state
        .scoped(Some(workspace_id), Some(member_id))
        .await
        .unwrap();
    sqlx::query("insert into memberships (user_id, workspace_id, role) values ($1, $2, 'member')")
        .bind(member_id)
        .bind(workspace_id)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let (status, headers, _) = send(
        &router,
        json_request(
            "POST",
            "/api/v1/auth/login",
            json!({ "email": member_email, "password": "password123" }),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let member_cookie = session_cookie(&headers);

    let uri = format!("/api/v1/workspaces/{workspace_id}/invites");
    let invite_body = json!({ "email": "invitee@example.test", "role": "member" });

    let (status, _, body) = send(
        &router,
        json_request("POST", &uri, invite_body.clone(), Some(&member_cookie)),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "member must not create invites"
    );
    assert_eq!(body["error"]["code"], "forbidden");

    let (status, _, body) = send(
        &router,
        json_request("POST", &uri, invite_body, Some(&owner_cookie)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "owner must be able to invite");
    assert!(
        body["token"]
            .as_str()
            .map(|t| t.len() >= 20)
            .unwrap_or(false),
        "invite should carry an opaque token"
    );
}

/// (a) RLS blocks cross-tenant reads and denies by default.
#[tokio::test]
async fn rls_blocks_cross_tenant_reads() {
    let (_router, state) = setup().await;

    let user_a = insert_user(&state, "a").await;
    let user_b = insert_user(&state, "b").await;
    let workspace_a = create_workspace(&state, user_a, "A").await;
    let workspace_b = create_workspace(&state, user_b, "B").await;

    // Tenant B must not be able to read tenant A's workspace...
    let mut tx = state.scoped(Some(workspace_b), Some(user_b)).await.unwrap();
    let leaked: Option<Uuid> = sqlx::query_scalar("select id from workspaces where id = $1")
        .bind(workspace_a)
        .fetch_optional(&mut *tx)
        .await
        .unwrap();
    assert!(leaked.is_none(), "tenant B read tenant A's workspace");

    // ...nor A's memberships.
    let leaked_memberships: i64 =
        sqlx::query_scalar("select count(*) from memberships where workspace_id = $1")
            .bind(workspace_a)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(
        leaked_memberships, 0,
        "tenant B read tenant A's memberships"
    );

    // With no tenant context at all, RLS must hide every workspace.
    let mut tx = state.scoped(None, None).await.unwrap();
    let visible: i64 = sqlx::query_scalar("select count(*) from workspaces")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(visible, 0, "missing tenant context must deny all rows");
}

async fn insert_user(state: &AppState, tag: &str) -> Uuid {
    let email = format!("rls-{tag}-{}@example.test", Uuid::new_v4());
    sqlx::query_scalar(
        "insert into users (email, name, password_hash) values ($1::citext, $2, $3) returning id",
    )
    .bind(email)
    .bind(tag)
    .bind("not-a-real-hash")
    .fetch_one(&state.pool)
    .await
    .unwrap()
}

async fn create_workspace(state: &AppState, user_id: Uuid, name: &str) -> Uuid {
    let workspace_id = Uuid::new_v4();
    let mut tx = state
        .scoped(Some(workspace_id), Some(user_id))
        .await
        .unwrap();
    sqlx::query("insert into workspaces (id, name, plan) values ($1, $2, 'free')")
        .bind(workspace_id)
        .bind(name)
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("insert into memberships (user_id, workspace_id, role) values ($1, $2, 'owner')")
        .bind(user_id)
        .bind(workspace_id)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    workspace_id
}
