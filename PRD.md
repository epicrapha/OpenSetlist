# OpenSetlist: Product Requirements Document

Version: 0.1 (Draft)
Status: Ready for review
Product type: Open source core with an optional hosted service
Primary platform: Progressive Web App (PWA)

---

## 1. Overview

OpenSetlist is a setlist builder and music team management tool for Christian worship teams. It helps worship leaders build setlists from a shared song library, share them with their band, and perform from them on stage, even without an internet connection.

The long term product also covers events, calendars, programmes, teams and roles, and a homepage dashboard. The MVP focuses on the song library and the setlist builder, because that is the core daily job of a worship leader.

The project is built on Svelte (SvelteKit) for the frontend and Rust for the backend, chosen for fast load times, a small footprint, and easy self-hosting.

The project is open source at its core so any church can self-host it for free. A hosted version offers the same product without the setup work, funded by paid plans.

## 2. Problem Statement

Worship leaders usually plan services with a mix of tools: group chats, spreadsheets, shared drives, PDFs, and screenshots of chord charts. This causes real problems:

- Songs, keys, and chord charts are scattered and often out of date.
- Band members do not know the key, order, or arrangement until the last minute.
- Transposing a chart for a different singer is manual and error prone.
- Existing worship planning tools are often expensive, closed, or heavy for small churches.
- Many tools fail on stage when the Wi-Fi is weak or missing.

The open source options that do exist have their own gaps. Many open source setlist builders fall short in two areas:

- **Poor user interface:** Cluttered, dated, or confusing screens that are hard to use quickly, especially on a phone or tablet in a dim room. Volunteers with little time give up on them.
- **Low quality and weak security:** A growing number of projects are rushed together with AI generated code that was never properly reviewed, tested, or hardened. This can leave real vulnerabilities such as weak authentication, missing permission checks, and exposed user data. Churches store member names, contact details, and schedules, so this is a serious risk, and most churches have no one on staff who can audit the code.

OpenSetlist aims to be a free, open, fast, and reliable alternative that works well for small and medium churches first.

## 3. Goals and Non-Goals

### 3.1 Goals

1. Let a worship leader build a complete setlist in under five minutes using songs from a shared library.
2. Make chord charts easy to store, view, transpose, and share with the team.
3. Work reliably offline so a team can perform from a saved setlist with no connection.
4. Give the team a live, shared view so everyone sees the same setlist and changes in real time.
5. Offer both a self-hosted open source option and a hosted option from the same codebase.
6. Lay a clean data foundation for events, calendars, programmes, teams, and roles in later releases.

### 3.2 Non-Goals (MVP)

- Slide and lyric projection software for the congregation.
- Audio mixing, in-ear monitor control, or live streaming tools.
- Payments, donations, or church accounting.
- A public song marketplace or licensed lyric database.
- Native iOS and Android apps (the PWA covers mobile for now).

## 4. Target Users

### 4.1 Primary persona: Worship Leader / Music Director

Plans the service music, picks songs and keys, builds the setlist, and leads rehearsal. Wants speed, control, and confidence on stage. Comfortable with a phone or tablet, not necessarily technical.

### 4.2 Secondary personas (post-MVP and light MVP access)

- **Band member or vocalist:** Needs to see the setlist, keys, and charts for their part, and learn songs ahead of time.
- **Pastor or church admin:** Wants to see what is planned for each service and approve or review the programme.
- **Tech and sound team:** Needs the running order and notes for transitions.
- **Self-hosting administrator:** Installs, configures, and maintains an instance for their church.

## 5. Product Principles

1. **Stage first.** Every screen that may be used live must be readable, fast, and hard to break.
2. **Offline by default.** Core flows must work without a connection and sync later.
3. **Simple for volunteers.** Most users are volunteers with little time. Fewer taps, plain language.
4. **Open and portable.** Users own their data and can export it in open formats.
5. **Respect copyright.** Make it easy to stay legal and never encourage unlicensed sharing.

## 6. Scope and Phasing

### 6.1 MVP (Phase 1): Song library and setlist builder

- Account sign up, sign in, and a single church workspace.
- Song library with ChordPro and plain text chord charts.
- Transpose and capo tools.
- Audio and video reference links (YouTube, Spotify, and generic URLs).
- Sheet music PDF uploads attached to songs.
- Setlist builder with drag and drop ordering, per-song key, and notes.
- Stage view (performance mode).
- Real-time collaboration on setlists.
- Installable PWA with offline access to saved setlists and songs.
- Basic invite flow so band members can view setlists.
- Export and print of setlists and charts.

### 6.2 Phase 2: Teams, roles, and scheduling

- Teams (for example Worship Team A, Youth Band).
- Roles and positions (vocals, keys, drums, sound, and so on).
- Member availability and service scheduling.
- Accept and decline requests with notifications.

### 6.3 Phase 3: Events, calendar, and programmes

- Events (Sunday services, rehearsals, special events).
- Calendar views (month, week, agenda) with calendar feed export.
- Programmes (full order of service including non-music items).
- Attach setlists and teams to events.

### 6.4 Phase 4: Dashboard and integrations

- Homepage dashboard with upcoming events, assigned roles, recent setlists, and quick actions.
- Planning Center and other import and export tools.
- CCLI and usage reporting support.
- Multi-campus and multi-church features for the hosted service.

## 7. Functional Requirements

Priority key: **P0** must have for MVP, **P1** should have, **P2** nice to have.

### 7.1 Accounts and Workspaces

| ID | Requirement | Priority |
|----|-------------|----------|
| AUTH-1 | Users can sign up and sign in with email and password. | P0 |
| AUTH-2 | Users can sign in with a magic link or an OAuth provider. | P1 |
| AUTH-3 | A workspace represents one church or organisation. | P0 |
| AUTH-4 | Workspace owners can invite members by link or email. | P0 |
| AUTH-5 | Users can belong to more than one workspace (hosted). | P2 |
| AUTH-6 | Self-hosted instances can disable public sign up. | P1 |

### 7.2 Song Library

| ID | Requirement | Priority |
|----|-------------|----------|
| SONG-1 | Create, edit, duplicate, and delete songs with title, artist, default key, tempo, time signature, and tags. | P0 |
| SONG-2 | Store chord charts in ChordPro format with a live preview. | P0 |
| SONG-3 | Accept plain text charts where chords sit above lyrics and convert them to ChordPro. | P0 |
| SONG-4 | Transpose a chart to any key, with sharps or flats preference. | P0 |
| SONG-5 | Capo tool that shows the chord shapes for a chosen capo position. | P0 |
| SONG-6 | Attach reference links (YouTube, Spotify, and other URLs) to a song. | P0 |
| SONG-7 | Upload sheet music PDFs and attach them to a song, with a size limit. | P0 |
| SONG-8 | Support multiple arrangements or versions of one song (for example acoustic or extended). | P1 |
| SONG-9 | Search and filter by title, artist, key, tempo, tag, and last used date. | P0 |
| SONG-10 | Import songs from ChordPro files and plain text files in bulk. | P1 |
| SONG-11 | Export songs as ChordPro, PDF, or plain text. | P1 |
| SONG-12 | Show usage history for each song (when and how often it was played). | P1 |
| SONG-13 | Section markers for verse, chorus, bridge, and so on, used for navigation. | P1 |

### 7.3 Setlist Builder

| ID | Requirement | Priority |
|----|-------------|----------|
| SET-1 | Create a setlist with a title, date, and optional service type. | P0 |
| SET-2 | Add songs from the library using search, with quick add. | P0 |
| SET-3 | Reorder songs with drag and drop, with a touch friendly alternative. | P0 |
| SET-4 | Set a per-song key, capo, and notes that override the library default without changing it. | P0 |
| SET-5 | Add non-song items such as prayer, scripture reading, offering, or transition notes. | P1 |
| SET-6 | Show total estimated duration based on song length. | P1 |
| SET-7 | Duplicate a setlist, and save a setlist as a reusable template. | P1 |
| SET-8 | Share a setlist by link with view-only access. | P0 |
| SET-9 | Print or export a setlist as PDF, including charts. | P0 |
| SET-10 | Flag key changes and big tempo jumps between songs as friendly hints. | P2 |
| SET-11 | Setlist status: draft, ready, and archived. | P1 |

### 7.4 Stage View (Performance Mode)

| ID | Requirement | Priority |
|----|-------------|----------|
| STAGE-1 | Full screen view that shows one song at a time with large, readable chords and lyrics. | P0 |
| STAGE-2 | Next and previous navigation by tap, swipe, or keyboard and Bluetooth foot pedal keys. | P0 |
| STAGE-3 | Adjustable font size, theme (light, dark, high contrast), and column layout. | P0 |
| STAGE-4 | Quick key change during the song without leaving stage view. | P1 |
| STAGE-5 | Auto-scroll with adjustable speed. | P1 |
| STAGE-6 | Keep the screen awake while stage view is open. | P0 |
| STAGE-7 | Show the PDF sheet music if no chart is available. | P1 |
| STAGE-8 | Leader can push the current song to all band devices in a live session. | P1 |

### 7.5 Real-Time Collaboration

| ID | Requirement | Priority |
|----|-------------|----------|
| COLLAB-1 | Multiple editors can work on the same setlist at once and see changes within about one second. | P0 |
| COLLAB-2 | Show who else is viewing or editing and where. | P1 |
| COLLAB-3 | Resolve conflicting edits automatically without data loss. | P0 |
| COLLAB-4 | Edits made offline merge cleanly when the device reconnects. | P0 |
| COLLAB-5 | Comments on a setlist or a song within a setlist. | P2 |
| COLLAB-6 | Activity log showing who changed what and when. | P1 |

### 7.6 Offline and PWA

| ID | Requirement | Priority |
|----|-------------|----------|
| PWA-1 | App is installable on iOS, Android, and desktop. | P0 |
| PWA-2 | Songs, charts, and setlists used recently are cached for offline use. | P0 |
| PWA-3 | Users can pin a setlist to make sure it is fully available offline, including PDFs. | P0 |
| PWA-4 | Clear indicator of online, offline, and syncing state. | P0 |
| PWA-5 | Offline edits are queued and synced with conflict handling. | P0 |
| PWA-6 | Show storage usage and let users clear cached data. | P1 |

### 7.7 Teams and Roles (Phase 2)

- Create teams inside a workspace and add members to them.
- Define positions (vocals, acoustic guitar, keys, drums, bass, sound, slides).
- Workspace roles: Owner, Admin, Leader, Member, Viewer.
- Assign people to positions on a specific service and track their response (pending, accepted, declined).
- Member profiles with instruments, vocal range, and default keys.
- Blockout dates and availability.

### 7.8 Events and Calendar (Phase 3)

- Create one-time and recurring events with type, location, start, and end.
- Attach a setlist, a programme, and a team to an event.
- Calendar views for month, week, and agenda.
- Calendar feed (iCal) for personal calendar apps.
- Reminders by email and push notification.

### 7.9 Programmes (Phase 3)

- A programme is the full order of service for an event, with timed items.
- Item types: song block (linked to a setlist), scripture, prayer, sermon, announcement, offering, and custom.
- Assign an owner to each item.
- Reorder items, set durations, and show running time totals.
- Printable and shareable programme view.

### 7.10 Homepage Dashboard (Phase 4)

- Next upcoming event and its setlist.
- My assignments and pending requests.
- Recently edited setlists and recently used songs.
- Quick actions: new setlist, add song, new event.
- Team activity feed.
- Different widgets by role (leader, band member, admin).

## 8. Roles and Permissions

| Capability | Owner | Admin | Leader | Member | Viewer |
|-----------|:-----:|:-----:|:------:|:------:|:------:|
| Manage workspace settings and billing | Yes | No | No | No | No |
| Invite and remove members | Yes | Yes | No | No | No |
| Create and edit songs | Yes | Yes | Yes | Suggest only | No |
| Delete songs | Yes | Yes | Yes | No | No |
| Create and edit setlists | Yes | Yes | Yes | No | No |
| View setlists and charts | Yes | Yes | Yes | Yes | Yes |
| Use stage view | Yes | Yes | Yes | Yes | Yes |
| Manage teams and schedule (Phase 2) | Yes | Yes | Yes | No | No |
| Respond to service requests (Phase 2) | Yes | Yes | Yes | Yes | No |

## 9. Non-Functional Requirements

### 9.1 Performance

- Initial load under 3 seconds on a mid-range phone on a 4G connection.
- Opening a cached song or setlist offline in under 500 ms.
- Real-time updates reach other users in about one second on a normal connection.
- Transposing a chart is instant with no visible delay.

### 9.2 Reliability

- Stage view must never lose data or crash if the connection drops.
- Offline data survives app restarts and browser updates where the platform allows.
- Daily backups on the hosted service and documented backup steps for self-hosters.

### 9.3 Usability and Accessibility

- Meet WCAG 2.1 AA for contrast, focus, and labels.
- Touch targets large enough for use on stage and in low light.
- Works on phones, tablets, and desktops, with a tablet-friendly stage view.
- Support for right to left and multiple languages in the interface over time.

### 9.4 Security and Privacy

- Passwords stored with a modern hashing algorithm, and optional two-factor sign in.
- All traffic over HTTPS.
- Workspace data isolated by tenant, with role checks on every request.
- Users can export and delete their own data.
- Minimal personal data collected, and no ads or data selling.

### 9.5 Open Source and Portability

- Core product released under a permissive or copyleft open source license, to be decided (see open questions).
- Self-hosting via Docker with a clear setup guide.
- Open data formats: ChordPro, JSON, iCal, and PDF.
- Hosted-only features are kept in a clearly separated module.

## 10. Copyright and Licensing Considerations

Worship songs are usually protected by copyright. OpenSetlist should help churches stay compliant and must not make unlicensed sharing easy.

- The product does not ship with copyrighted lyrics or charts.
- Users add their own content, and the terms of use state that users are responsible for having the right licences.
- Show a reminder when uploading or sharing charts.
- Provide a field for copyright and licence information on each song (including CCLI song number).
- Public sharing links expose only what the owner chooses, and default to team-only access.
- Plan for usage reporting for licence holders in a later phase.

## 11. Technical Approach (Recommended, for discussion)

The chosen stack is SvelteKit for the frontend and Rust for the backend. This section is guidance for the engineering plan and details can still change.

### 11.1 Stack Summary

| Layer | Choice | Reason |
|-------|--------|--------|
| Frontend | SvelteKit (static SPA mode) as an installable PWA | Small, fast bundles for phones and weak church Wi-Fi, strong service worker support, large contributor pool. |
| Backend | Rust with Axum | Small, fast, memory-light server. One binary plus a database can run on a cheap VPS or a Raspberry Pi. |
| Database | PostgreSQL for the hosted service, SQLite supported for self-hosting | Lets a church run the whole app without a separate database server. |
| Local (browser) storage | IndexedDB | Offline-first access to songs, charts, and setlists. |
| Real-time sync | Yjs on the client with Yrs on the Rust server, over WebSockets | Mature CRDT tooling that merges offline edits safely. |
| File storage | S3 compatible storage, with local disk option for self-hosting | Stores sheet music PDFs. |
| API contract | OpenAPI spec with generated TypeScript types | Keeps the Svelte and Rust sides in sync despite two languages. |
| Deployment | Docker Compose for self-hosting, managed infrastructure for hosted | One codebase for both distribution models. |

### 11.2 Frontend Notes

- Service worker handles caching, offline pinning of setlists, and background sync of queued edits.
- IndexedDB is the local source of truth while offline, and syncs with the server when a connection returns.
- Stage view uses the Screen Wake Lock API and keyboard events to support Bluetooth foot pedals.
- Drag and drop needs a touch friendly implementation and an alternative control for accessibility.
- Keep the initial bundle small and measure it against the performance targets in section 9.1.

### 11.3 Backend Notes

- Axum REST API for accounts, workspaces, songs, setlists, and uploads, with role checks on every request.
- WebSocket endpoint for live collaboration, backed by Yrs for merging document updates.
- Background job queue for exports, email, and file processing.
- Support both PostgreSQL and SQLite through a database layer that avoids vendor specific features where practical.
- Aim to ship the backend as a single binary plus static frontend assets.

### 11.4 Chord Engine

- Start with the ChordPro parser, transpose, and capo logic in TypeScript so it runs directly in the browser and offline.
- Cover it with a large set of real-world test charts.
- Move it into a shared Rust crate compiled to WebAssembly only if there is a clear need, such as a server-side use case or performance issue. This keeps a single source of truth if that happens.

### 11.5 Multi-Tenancy

- Tenant ID on all records, used by both the self-hosted and hosted versions so there is one codebase.
- Hosted-only features live in a clearly separated module.

### 11.6 Tradeoffs Accepted

- Two languages means more setup work and a need to keep types in sync through generated clients.
- Rust compile times and learning curve can slow early iteration, so keep the backend simple and well structured at first.
- Fewer ready-made Rust libraries for auth, email, and admin tooling, so budget time to assemble or build these.

## 12. Core Data Model (Draft)

- **Workspace:** id, name, settings, plan.
- **User:** id, name, email, preferences.
- **Membership:** user, workspace, role.
- **Song:** id, workspace, title, artist, default key, tempo, time signature, tags, licence info, ChordPro body.
- **Arrangement:** id, song, name, key, ChordPro body.
- **SongAsset:** id, song, type (link or PDF), URL or file, label.
- **Setlist:** id, workspace, title, date, status, event (optional).
- **SetlistItem:** id, setlist, position, type (song, note, custom), song or arrangement, key override, capo override, notes.
- **Team, Position, Assignment:** for Phase 2.
- **Event, Programme, ProgrammeItem:** for Phase 3.
- **ActivityLog:** id, workspace, actor, entity, change, timestamp.

## 13. Key User Flows

### 13.1 Build a Setlist (MVP)

1. Leader opens OpenSetlist and taps New Setlist.
2. Leader names the setlist and sets the date.
3. Leader searches the library and adds songs.
4. Leader reorders songs and sets the key for each one.
5. Leader adds notes for transitions.
6. Leader shares the link with the band.
7. Band members open the setlist and review keys and charts.

### 13.2 Perform From Stage (MVP)

1. Leader pins the setlist for offline use before the service.
2. On stage, leader opens stage view.
3. Leader moves through songs with swipes or a foot pedal.
4. If needed, leader changes a key on the spot.
5. The band follows along, in sync, if live session mode is on.

### 13.3 Add a Song (MVP)

1. User taps Add Song.
2. User pastes a plain text chart or ChordPro text.
3. App converts and previews the chart.
4. User sets the default key and tempo, and adds a reference link.
5. User uploads a sheet music PDF, if available.
6. User saves the song to the library.

## 14. Success Metrics

### 14.1 MVP Metrics

- Time to build a first setlist: median under 5 minutes.
- Activation: 60 percent of new workspaces create at least one setlist in their first week.
- Retention: 40 percent of active workspaces build a setlist in 4 of their first 6 weeks.
- Offline reliability: fewer than 1 percent of stage sessions report a failure.
- Collaboration: 50 percent of workspaces have at least two people viewing or editing a setlist.

### 14.2 Open Source Health

- GitHub stars, forks, and contributors.
- Number of self-hosted installs (opt-in telemetry only).
- Time to first response on issues and pull requests.

### 14.3 Hosted Business Metrics

- Conversion from free to paid plans.
- Monthly recurring revenue and churn.
- Support load per workspace.

## 15. Open Source and Hosted Model

- **Open source core:** Song library, setlists, stage view, collaboration, PWA, teams, events, and programmes, all free to self-host.
- **Hosted service:** Managed hosting, backups, updates, support, and email and push delivery handled for the church.
- **Possible paid features (hosted):** Larger storage, multi-campus management, advanced reporting, priority support, and integrations that cost money to run.
- **Principle:** Nothing needed to run a church week to week should be locked behind the hosted plan.

## 16. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Offline sync conflicts cause data loss | High | Use a proven CRDT approach, test heavily, keep version history. |
| Copyright complaints about hosted content | High | Clear terms, no bundled lyrics, takedown process, private by default. |
| iOS PWA limits (storage, background sync) | Medium | Test early on iOS, document limits, consider a native wrapper later. |
| Scope creep into a full church management system | Medium | Keep MVP strictly to library and setlists, phase the rest. |
| Hard to support self-hosters | Medium | Docker first setup, good docs, community forum. |
| Volunteer maintainers burn out | Medium | Clear contribution guide and a small, well-tested core. |
| Chord parsing edge cases | Medium | Build a shared, well-tested chord library with real-world sample charts. |

## 17. Milestones (Suggested)

| Milestone | Contents |
|-----------|----------|
| M0: Foundations | Repo, license, design system, auth, workspace, CI, Docker setup. |
| M1: Song library | Songs, ChordPro editor, transpose and capo, links, PDF uploads, search. |
| M2: Setlist builder | Setlists, ordering, overrides, share links, export and print. |
| M3: Stage and offline | Stage view, PWA install, offline caching and pinning, offline sync. |
| M4: Collaboration | Real-time editing, presence, live session mode, activity log. |
| M5: MVP beta | Pilot with 5 to 10 churches, bug fixing, docs, hosted beta. |
| M6: Public launch | Open source release and hosted launch. |
| Later | Teams and roles, events and calendar, programmes, dashboard, integrations. |

## 18. Open Questions

1. Which open source license should the core use (MIT, Apache 2.0, AGPL)? This affects how the hosted business can be protected.
2. What is the target size of a church for the hosted plan, and what is the free tier limit?
3. Should song data be shareable between workspaces, for example a community library of user-contributed chord charts, and how would that be moderated for copyright?
4. Which languages should the interface support at launch?
5. Is there a need for member-level private notes on songs, such as a drummer's own notes?
6. Should the hosted service support single sign-on for larger churches?
7. Which integrations (Planning Center, CCLI, Google Calendar) should come first after the MVP?
8. How should audio references behave if a streaming link is removed or restricted in a region?

## 19. Appendix: Glossary

- **ChordPro:** A plain text format for writing chords inline with lyrics.
- **Setlist:** The ordered list of songs, with keys and notes, for one service or event.
- **Programme:** The full order of service, including non-music items.
- **Stage view:** A large, simple full screen view used while performing.
- **Capo:** A device that raises the pitch of a guitar, which changes the chord shapes played.
- **PWA:** Progressive Web App, a website that can be installed and used offline.
- **CRDT:** A data structure that lets several people edit at once and merge changes without conflicts.
