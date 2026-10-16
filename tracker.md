# OpenSetlist — Live Tracker

Real-time status board. **Task IDs match `tasks.md`** — that file describes scope, this one tracks state. Updated as work lands, not in batches.

**Legend:** ⬜ todo · 🔄 in-progress · 🚧 blocked · 👀 review · ✅ done · ❌ cancelled
**Updated:** 2026-10-16

---

## Current focus

**Milestone:** M1 — Song library · **Active:** M1-C chord engine (`ses_ebb7b599…`) ∥ M1-S server songs (`ses_ebb7b5a8…`) · **Frozen:** song API contract in `PLAN.md` §3.1 · **Blocked:** M1-W web UI (needs M1-C’s frozen chord API + the server endpoints).

---

## M0 — Foundations

| ID | Task | Lane | Status | Evidence / note |
|---|---|---|---|---|
| M0-I1 | Repo scaffold (`.gitignore`, `.env.example`) | L3 orch | ✅ done | both written |
| M0-I2 | `LICENSE` AGPL-3.0 + `README.md` | L3 orch | ✅ done | README + full AGPL-3.0 text (via web tool) |
| M0-I3 | `docker-compose.yml` | L3 orch | ✅ done | postgres + server + web |
| M0-I4 | CI workflow | L3 orch | ✅ done | `.github/workflows/ci.yml` |
| M0-I5 | `PLAN.md` + `tasks.md` + `tracker.md` | L3 orch | ✅ done | `PLAN.md`, `tasks.md` written this turn |
| M0-S1 | Axum skeleton (config/tracing/health/error) | L1 fixer | ✅ done | lane done; fmt + clippy clean |
| M0-S2 | SQLx + PgPool + migrations | L1 fixer | ✅ done | migrations 0001/0002 + `sqlx migrate` |
| M0-S3 | Auth (argon2id + tower-sessions) | L1 fixer | ✅ done | argon2 + tower-sessions; register/login tests pass |
| M0-S4 | Workspaces + memberships + roles | L1 fixer | ✅ done | |
| M0-S5 | Tenant middleware + RLS | L1 fixer | ✅ done | `SET LOCAL ROLE` + tx-scoped `set_config` GUC |
| M0-S6 | Invite flow | L1 fixer | ✅ done | `non_admin_cannot_create_invites` passes |
| M0-S7 | RLS isolation + role-guard tests | L1 fixer | ✅ done | fixed: per-test pool (was a static shared across per-test runtimes); 3/3 pass parallel + serial |
| M0-S8 | Server Dockerfile + SQLx offline | L1 fixer | ✅ done | image builds (142 MB); `sqlx::migrate!` embeds migrations |
| M0-W1 | SvelteKit scaffold (SPA, Tailwind) | L2a fixer | ✅ done | SvelteKit 3.1.0; check 0/0, build ok (orch-verified) |
| M0-W2 | Design system | L2b designer | ✅ done | oklch tokens in `layout.css`; polished (scroll-padding, underline offset, caret); orch-verified |
| M0-W3 | App shell + routing | L2b designer | ✅ done | `app/+layout.svelte`; polished (spinner, workable "Try again", `role=status`); orch-verified |
| M0-W5 | Auth pages + workspace create/select | L2b designer | ✅ done | polished (focusable error regions, empty state); orch-verified: check 0/0, build ok, lint clean |
| M0-W6 | PWA manifest + service worker | L2a fixer | ✅ done | manifest + sw.js emitted to `build/` |
| M0-W7 | Web Dockerfile | L2a fixer | ✅ done | image builds (104 MB); nginx proxies `/api/` + `/health` |
| M0-A1 | compose up end-to-end flow | L5 fixer | ✅ done | both images build; e2e via nginx :3000 PASS — orch re-ran `scripts/m0-acceptance.sh` |
| M0-A2 | Cross-tenant read blocked | L5 fixer | ✅ done | user B → **404** typed `not_found`, no row leak; also proven by `rls_blocks_cross_tenant_reads` |
| M0-A3 | clippy+test / check+build green | orch | ✅ done | server fmt/clippy clean + full suite green (2 lib, 3 integration); web check 0/0, build, lint |
| M0-A4 | PWA installs + offline shell | L6 fixer + orch | ✅ done | SW precaches `index.html` + `__vite__mapDeps` graph (15 refs, 0 missing); manifest `id`/theme aligned; head tags moved to `app.html`; served & verified — no browser install test possible |

## M1 — Song library

| ID | Task | Lane | Status | Evidence / note |
|---|---|---|---|---|
| M1-C1 | ChordPro parser → AST | M1-C fixer | 🔄 in-progress | `ses_ebb7b599…` chord-engine lane |
| M1-C2 | Transpose | M1-C fixer | 🔄 in-progress | same lane (±semitones, sharp/flat pref) |
| M1-C3 | Capo shapes | M1-C fixer | 🔄 in-progress | same lane |
| M1-C4 | Plain-text → ChordPro | M1-C fixer | 🔄 in-progress | same lane |
| M1-C5 | Chart test suite | M1-C fixer | 🔄 in-progress | vitest, real-world charts |
| M1-S1 | Songs CRUD | M1-S fixer | 🔄 in-progress | `ses_ebb7b5a8…` server lane |
| M1-S2 | Arrangements | M1-S fixer | 🔄 in-progress | same lane |
| M1-S3 | Reference links | M1-S fixer | 🔄 in-progress | same lane |
| M1-S4 | PDF upload | M1-S fixer | 🔄 in-progress | same lane — ≤10 MB, disk, random object names |
| M1-S5 | Search/filter | M1-S fixer | 🔄 in-progress | same lane |
| M1-S6 | Export ChordPro/PDF/text | M1-S fixer | 🔄 in-progress | chordpro/text now; PDF export is P1 |
| M1-W1 | Library list + search UI | M1-W designer | ⬜ todo | blocked on M1-C + server endpoints |
| M1-W2 | Chord editor + live preview | M1-W designer | ⬜ todo | blocked on M1-C |
| M1-W3 | Transpose + capo controls | M1-W designer | ⬜ todo | blocked on M1-C |
| M1-W4 | Reference links UI | M1-W designer | ⬜ todo | blocked on server |
| M1-W5 | PDF upload UI | M1-W designer | ⬜ todo | blocked on server |
| M1-W6 | Add-song flow | M1-W designer | ⬜ todo | blocked on both |
| M1-A1 | SONG-1…7, SONG-9 verified | orch | ⬜ todo | after M1-W |

## Later milestones

| Milestone | Status | Note |
|---|---|---|
| M2 Setlist builder | ⬜ todo | decompose when reached |
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
