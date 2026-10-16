# OpenSetlist — Implementation Plan

Companion to `PRD.md`. This is the working plan for the code, not the product spec.

## 0. Locked decisions

| Decision | Choice | Why |
|---|---|---|
| First build scope | **M0 + M1** vertical slice | Runnable end-to-end; verify the stack before scaling. |
| Database | **PostgreSQL only** (SQLite later) | Use RLS properly; avoid dual-dialect cost now. |
| Auth | **argon2 + `tower-sessions`** (httpOnly cookie) | Revocable server sessions; no hand-rolled JWT. |
| Real-time collab | **Deferred to M4**, planned separately | Highest risk, no dedicated skill, worst fit for current model. |
| License | **AGPL-3.0** | Protects the hosted business. |
| Frontend render | SvelteKit `adapter-static`, SPA, PWA | Matches PRD 11.1; small bundles, offline-capable. |

## 1. Toolchain (verified on this machine)

rustc/cargo 1.99 · node 26 · bun 1.4 · Docker 29 + Compose 5.6 · sqlite3.
**No `psql`** — Postgres runs in Docker (`docker compose up -d db`).

## 2. Repository layout (monorepo)

```
OpenSetlist/
├── PRD.md · brief.md · PLAN.md · LICENSE (AGPL-3.0) · README.md
├── .env.example · .gitignore · docker-compose.yml
├── .github/workflows/ci.yml
├── server/                     # Rust + Axum  (owns: server/**)
│   ├── Cargo.toml · Dockerfile · .sqlx/
│   ├── migrations/             # SQLx migrations
│   └── src/{main,config,state,error,db}.rs
│       ├── auth/               # register, login, logout, me, sessions
│       ├── workspace/          # create, get, invite, membership, roles
│       └── middleware/         # tenant context (RLS), role guard
└── web/                        # SvelteKit  (owns: web/**)
    ├── package.json · svelte.config.js · vite.config.ts · Dockerfile
    ├── static/{manifest.webmanifest, icons}
    └── src/
        ├── lib/api/            # typed API client (contract §3)
        ├── lib/chord/          # TS chord engine (M1)
        └── routes/             # /, /login, /register, /songs, /setlists
```

Root files are owned by the orchestrator. Each lane owns its subtree only — no cross-writes.

## 3. API contract (M0, frozen)

Base: `/api/v1`. JSON. Errors: `{ "error": { "code", "message" } }`. Auth via session cookie.

| Method | Path | Body / notes | Auth |
|---|---|---|---|
| GET | `/health` | `{ "status": "ok" }` | public |
| POST | `/api/v1/auth/register` | `{ email, password, name }` → creates user | public |
| POST | `/api/v1/auth/login` | `{ email, password }` → session cookie | public |
| POST | `/api/v1/auth/logout` | ends session | session |
| GET | `/api/v1/auth/me` | current user + memberships | session |
| POST | `/api/v1/workspaces` | `{ name }` → workspace + owner membership | session |
| GET | `/api/v1/workspaces/:id` | workspace detail | member |
| POST | `/api/v1/workspaces/:id/invites` | `{ email, role }` | owner/admin |

### 3.1 M1 — song library contract (frozen at M1 start)

Base `/api/v1`, session cookie, workspace-scoped via RLS. Role gates per PRD §8: **reads** need `viewer`+; **create/update/delete/duplicate** need `leader`+ (owner/admin/leader). Members/viewers get 403 on writes.

**Song JSON**
```json
{ "id":"uuid", "workspace_id":"uuid", "title":"str", "artist":"str|null",
  "default_key":"str|null", "tempo":120, "time_signature":"4/4", "tags":["str"],
  "ccli_number":"str|null", "copyright":"str|null", "chordpro":"str",
  "created_at":"ts", "updated_at":"ts" }
```

| Method | Path | Notes |
|---|---|---|
| GET | `/api/v1/songs` | `?q=&key=&artist=&tag=&sort=title\|updated_at&limit=&offset=` → `{ "songs":[Song], "total":n }` |
| POST | `/api/v1/songs` | `{title, artist?, default_key?, tempo?, time_signature?, tags?, ccli_number?, copyright?, chordpro}` → 201 Song |
| GET | `/api/v1/songs/:id` | → Song; 404 when absent or another workspace's |
| PATCH | `/api/v1/songs/:id` | partial fields → Song |
| DELETE | `/api/v1/songs/:id` | → 204 |
| POST | `/api/v1/songs/:id/duplicate` | → 201 Song |
| GET | `/api/v1/songs/:id/export?format=chordpro\|text` | `text/plain` (SONG-11) |

**Arrangements** (SONG-8, P1): `GET|POST /api/v1/songs/:id/arrangements`, `PATCH|DELETE /api/v1/arrangements/:id`. Shape `{id, song_id, name, key, chordpro}`.

**Assets** (SONG-6 links, SONG-7 PDF): `POST /api/v1/songs/:id/assets` `{kind:"link", url, label?}` · `POST /api/v1/songs/:id/assets/pdf` multipart field `file` (≤10 MB, `application/pdf` only) · `DELETE /api/v1/assets/:id` · `GET /api/v1/assets/:id/file` (auth + tenant checked). Shape `{id, song_id, kind, url, label, filename, size_bytes, content_type, created_at}`.

Storage: disk under `UPLOAD_DIR` (default `./uploads`), **random** object names (never the client filename on disk); S3 later. Uploads are never statically served — only through the authenticated endpoint.


## 4. Data model (M0 slice)

- `users` — id, email (unique, citext), name, password_hash, created_at
- `sessions` — id, user_id, data, expiry (tower-sessions store)
- `workspaces` — id, name, settings jsonb, plan, created_at
- `memberships` — id, user_id, workspace_id, role enum(owner,admin,leader,member,viewer), unique(user_id,workspace_id)
- `invites` — id, workspace_id, email, role, token, expires_at, accepted_at

Every tenant table carries `workspace_id`; RLS policy uses `app.current_tenant_id`.

## 5. Milestones

### M0 — Foundations  *(this pass)*
Deliverables: repo + AGPL LICENSE + README + `.env.example` + CI; `docker-compose` (postgres, server, web); Axum skeleton (config, tracing, `/health`, graceful shutdown, error type); SQLx + migrations; auth (register/login/logout/me, argon2id, tower-sessions); workspace + membership + roles; tenant middleware + RLS; SvelteKit SPA scaffold + Tailwind design tokens + app shell + auth/workspace UI + PWA manifest/SW.
**Accept:** `docker compose up` → register → login → create workspace → `/auth/me` shows owner membership; RLS proven by a test that a second workspace cannot read the first's rows; `cargo test` + `npm run build` + lint pass; web installs as PWA.

### M1 — Song library  *(next pass)*
Chord engine (TS): ChordPro parse → AST, transpose (± semitones, sharp/flat pref, `Nashville`-free), capo shapes, plain-text→ChordPro; large unit-test suite of real charts. Server: song CRUD, arrangements, tags, tempo/key/time-sig, license fields (incl. CCLI no.), reference links, PDF upload (disk now, S3 later), search/filter. Web: library list/search, chord editor + live preview, transpose/capo controls, links, upload.
**Accept:** PRD SONG-1…SONG-7, SONG-9 pass; transpose/capo covered by tests; chart round-trips ChordPro export.

### M2 — Setlist builder · M3 — Stage + offline · M4 — Collaboration · M5 — Beta · M6 — Launch
Per PRD §17. M4 gets its own plan (Yjs/Yrs).

## 6. Build lanes (M0)

| Lane | Owner | Scope | Depends on |
|---|---|---|---|
| L1 server | @fixer (rust) | `server/**` | — |
| L2 web | @designer then @fixer | `web/**` | contract §3 |
| L3 infra | orchestrator | root files, CI, compose | — |

## 7. Skill / MCP mapping

- server → `axum-idioms`, `rust-sqlx-postgres-service`, `rust-service-security`, `rust-deployable-service`, `rust-engineer`
- tenancy/security → `saas-multi-tenant`, `api-security-best-practices`, `secure-code-guardian`, `security-reviewer`
- web → `svelte-runes`, `sveltekit-*`, `svelte-components/styling`
- design → `impeccable`, `ui-ux-pro-max`, `ui-styling`
- M1 chord engine → `typescript-pro`, `test-master`
- M3 → `pwa`, `offline-first-development`
- **MCP:** none connected (`context7` referenced but absent). Use webfetch/websearch for crate + library docs.

## 8. Verification strategy

Per lane: `cargo fmt --check && cargo clippy && cargo test` (server); `npm run check && npm run build` + vitest (web). Cross-lane: bring up compose, run the M0 acceptance flow. Security: RLS isolation test + tenant-check test (not inspection — executable proof).

## 9. Known risks on the current setup

All agents run one free flash model, including `oracle` and `security-reviewer` — so security review is **self**-review. Sections 9.4 and 2 of the PRD are the differentiator; treat this as the top risk and back security with executable tests, not judgement. CRDT (M4) remains the second risk and is deliberately out of scope here.
