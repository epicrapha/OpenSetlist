# OpenSetlist — Live Tracker

Real-time status board. **Task IDs match `tasks.md`** — that file describes scope, this one tracks state. Updated as work lands, not in batches.

**Legend:** ⬜ todo · 🔄 in-progress · 🚧 blocked · 👀 review · ✅ done · ❌ cancelled
**Updated:** 2026-10-16

---

## Current focus

**Milestone:** M1 — Song library · **Status: COMPLETE** (M1-C, M1-S, M1-W all green; acceptance **PASS**) · **Next:** M2 setlist builder (needs decomposition) · **Blocked:** none.

---

## M0 — Foundations

| ID | Task | Lane | Status | Evidence / note |
|---|---|---|---|---|
| M0-I1 | Repo scaffold (`.gitignore`, `.env.example`) | L3 orch | ✅ done | both written |
| M0-I2 | `LICENSE` AGPL-3.0 + `README.md` | L3 orch | ✅ done | README + full AGPL-3.0 text (via web tool) |
| M0-I3 | `docker-compose.yml` | L3 orch | ✅ done | postgres + server + web |
| M0-I4 | CI workflow | L3 orch | ✅ done | `.github/workflows/ci.yml` |
| M0-I5 | `PLAN.md` + `tasks.md` + `tracker.md` | L3 orch | ✅ done | |
| M0-S1 | Axum skeleton (config/tracing/health/error) | L1 fixer | ✅ done | lane done; fmt + clippy clean |
| M0-S2 | SQLx + PgPool + migrations | L1 fixer | ✅ done | migrations 0001/0002 + `sqlx migrate` |
| M0-S3 | Auth (argon2id + tower-sessions) | L1 fixer | ✅ done | argon2 + tower-sessions; register/login tests pass |
| M0-S4 | Workspaces + memberships + roles | L1 fixer | ✅ done | |
| M0-S5 | Tenant middleware + RLS | L1 fixer | ✅ done | `SET LOCAL ROLE` + tx-scoped `set_config` GUC |
| M0-S6 | Invite flow | L1 fixer | ✅ done | `non_admin_cannot_create_invites` passes |
| M0-S7 | RLS isolation + role-guard tests | L1 fixer | ✅ done | fixed: per-test pool (was a static shared across per-test runtimes); 3/3 pass parallel + serial |
| M0-S8 | Server Dockerfile + SQLx offline | L1 fixer | ✅ done | image builds (142 MB); `sqlx::migrate!` embeds migrations |
| M0-S9 | Invite accept + list endpoints | L1 fixer | ✅ done | `GET .../invites` (owner/admin 200, member 403, non-member **404**), `POST /invites/:token/accept` (404/409/403/201); `role:owner` rejected; migration `0004` token-scoped RLS policy (no BYPASSRLS). orch-verified: 20 tests, fmt/clippy clean |
| M0-W1 | SvelteKit scaffold (SPA, Tailwind) | L2a fixer | ✅ done | SvelteKit 3.1.0; check 0/0, build ok (orch-verified) |
| M0-W2 | Design system | L2b designer | ✅ done | oklch tokens in `layout.css`; polished (scroll-padding, underline offset, caret); orch-verified |
| M0-W3 | App shell + routing | L2b designer | ✅ done | `app/+layout.svelte`; polished (spinner, workable "Try again", `role=status`); orch-verified |
| M0-W5 | Auth pages + workspace create/select | L2b designer | ✅ done | polished (focusable error regions, empty state); orch-verified: check 0/0, build ok, lint clean |
| M0-W6 | PWA manifest + service worker | L2a fixer | ✅ done | manifest + sw.js emitted to `build/` |
| M0-W7 | Web Dockerfile | L2a fixer | ✅ done | image builds (104 MB); nginx proxies `/api/` + `/health` |
| M0-W8 | Invite UI: create/copy link + accept page | L2b designer | ✅ done | `/app/invites` (owner/admin only; create, copy link, pending/accepted/expired states) + public `/invite/[token]` accept page; `?next=` on login/register with an open-redirect guard. orch-verified: check 0/0, lint, build, **34 tests**, plus a headless-Chrome run of signed-out → register → auto-accept → workspace |
| M0-A1 | compose up end-to-end flow | L5 fixer | ✅ done | both images build; e2e via nginx :3000 PASS — orch re-ran `scripts/m0-acceptance.sh` |
| M0-A2 | Cross-tenant read blocked | L5 fixer | ✅ done | user B → **404** typed `not_found`, no row leak; also proven by `rls_blocks_cross_tenant_reads` |
| M0-A3 | clippy+test / check+build green | orch | ✅ done | server fmt/clippy clean + full suite green; web check 0/0, build, lint |
| M0-A4 | PWA installs + offline shell | L6 fixer + orch | ✅ done | SW precaches `index.html` + `__vite__mapDeps` graph (15 refs, 0 missing); manifest `id`/theme aligned; head tags moved to `app.html`; served & verified — no browser install test possible |
| M0-I6 | **Push M0 to GitHub** | L3 orch | ✅ done | `github.com/epicrapha/OpenSetlist` `origin/main` = `6ee0676`, 78 files, source-only |

## M1 — Song library

| ID | Task | Lane | Status | Evidence / note |
|---|---|---|---|---|
| M1-C1 | ChordPro parser → AST | M1-C fixer | ✅ done | `$lib/chord` — `parseChordPro` / `toChordPro`, typed `Chart` AST |
| M1-C2 | Transpose | M1-C fixer | ✅ done | `transpose` / `transposeKey` (±semitones, sharp/flat preference) |
| M1-C3 | Capo shapes | M1-C fixer | ✅ done | `capo` → `CapoResult` |
| M1-C4 | Plain-text → ChordPro | M1-C fixer | ✅ done | `fromPlainText` + `detectKey` |
| M1-C5 | Chart test suite | M1-C fixer | ✅ done | vitest **27/27** green |
| M1-S1 | Songs CRUD | M1-S fixer | ✅ done | `server/src/song/**`, migrations 0003; `song_crud_duplicate_and_export` passes |
| M1-S2 | Arrangements | M1-S fixer | ✅ done | `arrangements_lifecycle` passes |
| M1-S3 | Reference links | M1-S fixer | ✅ done | `links_and_pdf_assets` passes |
| M1-S4 | PDF upload | M1-S fixer | ✅ done | ≤10 MB + `application/pdf` enforced; random on-disk names; `links_and_pdf_assets` passes |
| M1-S5 | Search/filter | M1-S fixer | ✅ done | `GET /songs?q=&key=&artist=&tag=&sort=&limit=&offset=` |
| M1-S6 | Export ChordPro/text | M1-S fixer | ✅ done | `?format=chordpro\|text`; **PDF export deferred (P1)** |
| M1-S7 | List song assets (`GET /songs/:id/assets`) | M1-S fixer | ✅ done | orch-verified: shares `/songs/{id}/assets` with POST; tenant-scoped (`ensure_song`), deterministic order, no `object_name` leak; 13 server tests green |
| M1-W1 | Library list + search UI | M1-W designer | ✅ done | `app/songs/+page.svelte` — search, key/artist/tag filters, sort, pagination, empty/read-only/error states |
| M1-W2 | Chord editor + live preview | M1-W designer | ✅ done | `app/songs/[id]/+page.svelte` + `ChartView.svelte`; ChordPro source ↔ live parsed chart |
| M1-W3 | Transpose + capo controls | M1-W designer | ✅ done | view-only transforms (`transpose`/`capo`); stored `chordpro` never mutated |
| M1-W4 | Reference links UI | M1-W designer | ✅ done | add/delete/open links; list rendering pending M1-W7 |
| M1-W5 | PDF upload UI | M1-W designer | ✅ done | multipart upload, 413/415 surfaced, download + delete |
| M1-W6 | Add-song flow | M1-W designer | ✅ done | `app/songs/new/+page.svelte` — paste plain text/ChordPro → preview → save |
| M1-W7 | Load asset list from server | M1-W designer | ✅ done | `listAssets()` + detail page load/refresh from server; local-state workaround + "no endpoint" note removed; check 0/0, 30/30 |
| M1-A1 | SONG-1…7, SONG-9 verified | orch | ✅ done | `scripts/m1-acceptance.py` → **RESULT: PASS** (28 checks); role gate + RLS proven by `writes_require_leader_role` + isolation tests |

**M1 server verification (orch, on disk):** `cargo fmt --check` clean · `cargo clippy --all-targets -- -D warnings` clean · `cargo test` → 4 lib (incl. 2 song unit) + 3 security + **6 songs integration** all pass. Web: `npm run check` 0/0 · vitest 27/27.

## Later milestones

| Milestone | Status | Note |
|---|---|---|
| M2 Setlist builder | 🔄 planned | decomposed into M2-S1…S6 / W1…W9 / A1 (`tasks.md`); `PLAN.md` §3.3 to freeze before dispatch |
| M3 Stage + offline | ⬜ todo | |
| M4 Collaboration (CRDT) | ⬜ todo | own plan required |
| M5 Beta | ⬜ todo | |
| M6 Launch | ⬜ todo | |

---

## Blockers & risks

| # | Item | Impact | Status |
|---|---|---|---|
| B1 | `LICENSE` AGPL-3.0 text unfetchable from bash | Low | ✅ resolved — fetched via web tool |
| B2 | No MCP servers connected (`context7` absent) | Low | open |
| B3 | All agents run one free flash model incl. self-review | High | ongoing — mitigate with executable security tests |
| B4 | `rls_blocks_cross_tenant_reads` failed: `PoolTimedOut` — meta-static pool shared across per-test Tokio runtimes | High | ✅ resolved — pool rebuilt per test; 3/3 pass parallel + serial |
| B5 | oh-my-opencode preset gated `designer` skills to `[]`; global design skills were unreachable | Medium | resolved — granted in preset (needs restart) + copied into project `.opencode/skills/` |
| B6 | `SESSION_SECRET` (compose + `.env.example`) is unused — the server never reads it; tower-sessions falls back to a per-process cookie key | Medium | open — wire a persistent key before prod / multi-replica (M1+) |
| B7 | `sqlx::migrate!` rejects a DB that has an applied migration missing from `migrations/` — the live dev DB has 0003 applied, so a *M0-only* checkout cannot boot against it | Low | open — use a fresh DB (or `docker compose down -v`) when testing the M0 commit |
| B8 | **No accept-invite endpoint** — invites can be created but never accepted, so adding a teammate is impossible (M0-S6 is incomplete; `server/tests/songs.rs:361` inserts memberships directly) | Medium | ✅ resolved — accept + list endpoints, token-scoped RLS (`0004`), 404-for-non-members, and the full UI. Live API **18/18**; browser flow green |
| B9 | `GET /songs/:id/arrangements` returns a **bare array** while songs/assets return wrapped objects (`{songs,total}` / `{assets}`) | Low | open — align in `PLAN.md` §3.1 |

## Changelog

| Date | Change |
|---|---|
| 2026-10-16 | Created `PLAN.md`, `tasks.md`, `tracker.md`. Locked stack decisions. |
| 2026-10-16 | Root infra done: `.gitignore`, `.env.example`, `docker-compose.yml`, `README.md`, `.github/workflows/ci.yml`. Dispatched L1 server + L2a web scaffold. |
| 2026-10-16 | `LICENSE` (AGPL-3.0) added. Blocker B1 closed. L1 + L2a still running. |
| 2026-10-16 | L2a web scaffold complete + independently verified (check 0/0, build ok). W1/W4/W6/W7 done. Dispatched L2b design for W2/W3/W5. L1 server still running. |
| 2026-10-16 | L2b design landed + orch-verified (check 0/0, build, lint) → W2/W3/W5 done. Found designer ran without global design skills (preset `skills: []` + wrong path); preset fixed. Found RLS integration test failure (B4). Polish pass dispatched. |
| 2026-10-16 | Installed `impeccable`, `ui-ux-pro-max`, `ui-styling` into project `.opencode/skills/` (excluded impeccable's 20 MB linux-x64 browser binary). |
| 2026-10-16 | B4 fixed: `tests/security.rs` built a `PgPool` in a static reused across per-test runtimes → leaked permits → `PoolTimedOut`. Now one pool per test. Full server suite green (2 lib + 3 integration). |
| 2026-10-16 | Designer polish pass landed + orch-verified (check 0/0, build, lint); confirmed it read `impeccable`, `ui-ux-pro-max`, `ui-styling` (+`design`, `design-system`) by absolute path. M0-S*/M0-W* all done; M0-A1..A4 remain. |
| 2026-10-16 | M0-A3 satisfied (server suite + web check/build green). Dispatched two disjoint lanes: L5 `ses_ebb7b5a8…` (A1/A2 — compose build + HTTP e2e through nginx), L6 `ses_ebb7b599…` (A4 — PWA offline shell + manifest). |
| 2026-10-16 | **M0 complete.** A1/A2 verified end-to-end through compose+nginx (`scripts/m0-acceptance.sh`, orch independently re-ran → PASS); nginx gained a `/health` proxy (was hitting the SPA fallback). A4: SW now precaches the real bundle graph; manifest `id`/theme aligned; PWA head tags moved from `+layout.svelte` to `src/app.html` so the SPA fallback carries them; web image rebuilt + served artifacts verified. Opened B6. |
| 2026-10-16 | **M1 started.** Froze the song-library contract in `PLAN.md` §3.1 (songs/arrangements/assets, role gates, upload storage rules). Dispatched two disjoint lanes: M1-C chord engine (`web/src/lib/chord`, frozen public API) and M1-S server songs (`server/**`). M1-W UI held until M1-C lands. |
| 2026-10-16 | **M0 pushed to GitHub.** `github.com/epicrapha/OpenSetlist` (`origin/main` = `6ee0676`) — a clean M0-only commit: M1 WIP was moved aside and its edits to shared files (`server/src/{lib,config,error}.rs`, `Cargo.toml`, `tests/security.rs`, `web/{package.json,vite.config.ts}`, `package-lock.json`) reverted, M0 re-verified green on a fresh DB, committed, then M1 WIP restored. `.opencode/` (73 MB) gitignored. Opened B7. |
| 2026-10-16 | M1-C + M1-S confirmed **green on disk** (both lanes had aborted before reporting): server fmt/clippy clean, 4 lib + 3 security + 6 songs integration tests pass; chord engine vitest 27/27; web check 0/0. Tracker corrected from the stale "in-progress". Dispatched M1-W web UI lane. |
| 2026-10-16 | **M1-W done** (`app/songs/{+page,new,[id]}`, `ChartView`, `Select`, `Textarea`, `ArrangementForm`, `api/{songs,query,client,types}`, `download.ts`, `roles.ts`). Orch re-verified: check 0/0, build ok, lint clean, vitest **30/30**. Lane used `#lib` (not `$lib`) — codebase is uniformly `#lib` and SvelteKit 3 removed `$lib`; accepted. |
| 2026-10-16 | Gap found by M1-W: **no asset-list endpoint**, so links/PDFs disappeared on reload. Added `GET /api/v1/songs/:id/assets` to `PLAN.md` §3.1 + M1-S7/M1-W7 to the WBS; server and web lanes dispatched in parallel (disjoint trees). |
| 2026-10-16 | M1-S7 + M1-W7 landed; orch re-verified both (server 13 tests + fmt/clippy clean; web check 0/0, 30/30). Fixed the ephemeral-uploads gap the server lane flagged: `docker-compose.yml` now persists `UPLOAD_DIR=/data/uploads` in a named volume; `.env.example` documents the local default. |
| 2026-10-16 | **M1 acceptance PASS** — added `scripts/m1-acceptance.py` (28 checks): SONG-1 metadata, SONG-9 search/filter/sort/paginate, SONG-11 export (chordpro + text), duplicate, SONG-6 links, SONG-7 PDF (upload/download/415/413), SONG-8 arrangements lifecycle, auth (401/422), tenant isolation (404), delete. Opened B8, B9. |
| 2026-10-16 | **M1 pushed** (`bb85d50`) — rebased onto your `d24e51e` README edit, then pushed. |
| 2026-10-16 | Local dev servers started for manual testing (web :5173, server :8080, Postgres in Docker); stale M0 compose images stopped. Seeded demo account + one song. |
| 2026-10-16 | B8 in progress: froze `PLAN.md` §3.2 (invites). Server work landed + orch-verified (fmt/clippy clean, **20 tests**: 4 lib + 7 invites + 3 security + 6 songs). `0004_invite_accept.sql` adds a token-scoped RLS SELECT policy so accept can resolve a token *before* the tenant is known — RLS still enforced, no BYPASSRLS. Lane flagged non-members getting 403 on `GET .../invites` (leaks workspace existence); contract changed to **404**, fix + invite UI dispatched. |
| 2026-10-19 | **B8 closed.** Server: `POST /invites/:token/accept` + invite list, `0004_invite_accept.sql` (token-scoped RLS policy — no BYPASSRLS), 404 for non-members; 20 tests, live API smoke **18/18**. Web: `/app/invites` + public `/invite/[token]`, `?next=` with open-redirect guard; check 0/0, lint, build, **34 tests**, headless-Chrome flow signed-out → register → accept → workspace **PASS**. M2 decomposed in `tasks.md`; `PLAN.md` §3.3 (setlists) frozen. |
