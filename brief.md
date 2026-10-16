# OpenSetlist: Project Brief

**One line:** A free, open source setlist builder for worship teams that works on stage even with no Wi-Fi.

**Built on:** Svelte + Rust, for fast load times, a small footprint, and easy self-hosting.

---

## The Problem

Most worship teams plan their music with a messy mix of group chats, spreadsheets, shared drives, PDFs, and screenshots of chord charts. The result:

- Nobody is sure which key or order the songs are in until the last minute.
- Transposing a chart for a different singer is slow and easy to get wrong.
- Existing worship planning tools are often expensive or closed, which is hard for small churches.
- Many tools fall apart on stage when the connection is weak.

The open source options have their own problems too:

- **Poor UI:** Many open source setlist apps are cluttered, dated, and hard to use quickly on a phone or tablet.
- **Rushed, unreviewed code:** A lot of them are put together with AI generated code that was never properly reviewed or hardened, which can leave security holes like weak logins and missing permission checks. Churches store member names, contact details, and schedules, so this matters.

## The Idea

OpenSetlist gives a worship leader one place to keep their songs, build a setlist in minutes, share it with the band, and perform from it, online or offline.

## What the First Version Does

- **Song library:** Store chord charts in ChordPro or plain text, with reference links (YouTube, Spotify) and sheet music PDFs.
- **Transpose and capo tools:** Change the key instantly and see the right chord shapes.
- **Setlist builder:** Drag songs into order, set the key for each one, and add notes.
- **Stage view:** A big, clear, full screen performance mode that works with swipes or a Bluetooth foot pedal.
- **Real-time collaboration:** The whole team sees the same setlist and updates as they happen.
- **Works offline:** Installable as an app on any phone or tablet, with setlists saved for use without internet.

## What Comes Next

Teams and roles, scheduling, events and calendars, full service programmes, and a homepage dashboard for each person.

## Who It Is For

The main user is the worship leader or music director. Band members, pastors, and tech teams come in as the product grows.

## Why It Could Work

- **Free and open:** Any church can host it themselves, so small churches are not priced out.
- **Built for the stage:** Fast, readable, and reliable when it matters most.
- **Simple for volunteers:** Few taps, plain language.
- **Clear path to sustainability:** A hosted version for churches that want it managed for them, with paid plans for extras like more storage and support.

## How It Will Be Built

- **Frontend:** SvelteKit as an installable PWA, small and fast on phones.
- **Backend:** Rust with Axum, light enough to run on a cheap server.
- **Collaboration:** Yjs and Yrs so offline edits merge cleanly.
- **Hosting:** Docker for self-hosting, plus a managed hosted option.

## Business Model

Open source core that is free to self-host. A hosted plan sells convenience: managed hosting, backups, updates, and support. Nothing a church needs week to week is locked behind a paywall.

## Risks We Are Watching

- Offline sync conflicts, handled with proven collaboration tools and heavy testing.
- Song copyright, handled by not bundling lyrics, adding licence fields, and keeping sharing private by default.
- Scope creep, handled by keeping the first release focused on songs and setlists.

## What I Would Love From You

- Honest feedback on the idea and the features.
- Your thoughts on what worship teams you know struggle with most.
- If you are interested, help with testing, design, or building.

---

The full requirements are in PRD.md.
