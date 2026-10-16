# OpenSetlist — Task Map (WBS)

Decomposition of `PRD.md` / `PLAN.md` into work items. **Static**: describes *what* exists and how it links. Live status lives in `tracker.md`.

## Conventions

- **ID:** `M<milestone>-<area><n>` — area: `S`=server, `W`=web, `C`=chord engine, `I`=infra, `A`=acceptance.
- **Deps:** another ID, or `—`.
- **Ref:** PRD requirement / scenario it satisfies.
- Areas `S`/`W`/`I`/`C` map 1:1 to build lanes (see `PLAN.md` §6) and never share a write path.

---

## M0 — Foundations

### Infra (lane L3 · orchestrator) — owns root files
| ID | Task | Deps | Ref |
|---|---|---|---|
| M0-I1 | Repo scaffold: `.gitignore`, `.env.example` | — | — |
| M0-I2 | `LICENSE` (AGPL-3.0 full text) + `README.md` | — | §15 |
| M0-I3 | `docker-compose.yml`: postgres + server + web | I1 | — |
| M0-I4 | CI workflow: cargo fmt/clippy/test + web check/build | — | §17 M0 |
| M0-I5 | `PLAN.md` + `tasks.md` + `tracker.md` | — | — |

### Server (lane L1 · @fixer) — owns `server/**`
| ID | Task | Deps | Ref |
|---|---|---|---|
| M0-S1 | Cargo project + Axum skeleton: config, tracing, `/health`, graceful shutdown, typed error | — | §11.3 |
| M0-S2 | SQLx + `PgPool` + migration baseline | S1 | — |
| M0-S3 | Auth: `users` + argon2id + register/login/logout/me + `tower-sessions` Postgres store | S2 | AUTH-1 |
| M0-S4 | Workspaces + memberships + role enum | S3 | AUTH-3 |
| M0-S5 | Tenant middleware + Postgres RLS (`app.current_tenant_id`) | S4 | §11.5, §9.4 |
| M0-S6 | Invite flow (link/email, token, expiry) | S4 | AUTH-4 |
| M0-S7 | Security tests: RLS cross-tenant isolation + role guard (executable) | S5, S6 | §9.4 |
| M0-S8 | Multi-stage `Dockerfile` + SQLx offline prep | S2 | §11.3 |

### Web (lane L2 · @designer → @fixer) — owns `web/**`
| ID | Task | Deps | Ref |
|---|---|---|---|
| M0-W1 | SvelteKit scaffold: `adapter-static` SPA, TS, Tailwind, lint | — | §11.1 |
| M0-W2 | Design system: tokens, type scale, color, base components | W1 | §17 M0 |
| M0-W3 | App shell + layout + routing + nav | W2 | §5.3 |
| M0-W4 | Typed API client (contract §3) + session/401 handling | W1 | §11.1 |
| M0-W5 | Auth pages (register/login) + workspace create/select | W4 | AUTH-1, AUTH-3 |
| M0-W6 | PWA: manifest, service worker, icons | W1 | PWA-1 |
| M0-W7 | Static-build `Dockerfile` | W1 | §11.3 |

### M0 acceptance (orchestrator)
| ID | Task | Deps | Ref |
|---|---|---|---|
| M0-A1 | `docker compose up` → register → login → create workspace → `/auth/me` owner | S3,S4,W5 | §13 |
| M0-A2 | Cross-tenant read blocked (RLS) | S7 | §9.4 |
| M0-A3 | `cargo clippy`+`test` and `npm run check`+`build` green | S*,W* | §17 M0 |
| M0-A4 | PWA installs + offline shell loads | W6 | PWA-1 |

---

## M1 — Song library

### Chord engine (lane L4 · @fixer) — owns `web/src/lib/chord/**`
| ID | Task | Deps | Ref |
|---|---|---|---|
| M1-C1 | ChordPro parser → AST | — | SONG-2 |
| M1-C2 | Transpose (±semitones, sharp/flat preference) | C1 | SONG-4 |
| M1-C3 | Capo chord-shape mapping | C1 | SONG-5 |
| M1-C4 | Plain-text (chords-over-lyrics) → ChordPro | C1 | SONG-3 |
| M1-C5 | Unit suite over real-world charts | C1–C4 | §11.4 |

### Server (lane L1)
| ID | Task | Deps | Ref |
|---|---|---|---|
| M1-S1 | Songs CRUD + model (key, tempo, time-sig, tags, license/CCLI) | M0-S5 | SONG-1 |
| M1-S2 | Arrangements | S1 | SONG-8 |
| M1-S3 | Reference links (YouTube/Spotify/URL) | S1 | SONG-6 |
| M1-S4 | PDF upload (disk; S3 later) + size limit | S1 | SONG-7 |
| M1-S5 | Search/filter (title, artist, key, tempo, tag, last used) | S1 | SONG-9 |
| M1-S6 | Export ChordPro / PDF / plain text | S1,C1 | SONG-11 |
| M1-S7 | List song assets (`GET /songs/:id/assets`) — completes S3/S4 | S1 | SONG-6, SONG-7 |

### Web (lane L2)
| ID | Task | Deps | Ref |
|---|---|---|---|
| M1-W1 | Library list + search UI | M1-S5 | SONG-9 |
| M1-W2 | Chord editor + live preview | M1-C1 | SONG-2 |
| M1-W3 | Transpose + capo controls | M1-C2,C3 | SONG-4,5 |
| M1-W4 | Reference links UI | M1-S3 | SONG-6 |
| M1-W5 | PDF upload UI | M1-S4 | SONG-7 |
| M1-W6 | Add-song flow (paste → convert → preview → save) | C4,W2 | §13.3 |
| M1-W7 | Load asset list from server (replace local-state workaround) | M1-S7 | SONG-6, SONG-7 |

### M1 acceptance
| ID | Task | Deps | Ref |
|---|---|---|---|
| M1-A1 | SONG-1…7, SONG-9 pass; chord engine tests green; ChordPro round-trip | all M1 | §7.2 |

---

## Later milestones (epics — decompose when reached)

| Milestone | Epics | Ref |
|---|---|---|
| **M2** Setlist builder | setlists+items, drag/touch reorder, per-song key/capo/notes, share link, export/print | §7.3, §17 |
| **M3** Stage + offline | stage view, wake lock, PWA pinning, offline cache, sync queue, status indicator | §7.4, §7.6 |
| **M4** Collaboration | Yjs/Yrs CRDT, presence, live session push, activity log — **own plan** | §7.5 |
| **M5** Beta | pilot 5–10 churches, docs, hosted beta | §17 |
| **M6** Launch | OSS + hosted release | §17 |
| **Later** | teams/roles, events/calendar, programmes, dashboard, integrations | §6.2–6.4 |

## Open decisions (from PRD §18, not yet needed)

Free-tier limits · community song sharing + moderation · launch languages · member private notes · SSO · integration order · dead streaming links. None block M0/M1.
