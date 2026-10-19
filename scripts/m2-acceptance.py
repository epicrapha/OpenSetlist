#!/usr/bin/env python3
"""M2 acceptance — setlist builder, end-to-end over HTTP.

Covers PRD SET-1..SET-9, SET-11 plus auth, role gating and tenant isolation
(PLAN.md §3.3). SET-9 is a print stylesheet (no server endpoint) and SET-10
hints are client-side, so they are verified by the web checks, not here.

Usage:
    python3 scripts/m2-acceptance.py [BASE_URL]
    BASE_URL defaults to http://localhost:5173 (dev) — use :3000 for compose.
"""
import http.cookiejar
import json
import os
import sys
import urllib.error
import urllib.request
import uuid
from typing import Any

BASE = (sys.argv[1] if len(sys.argv) > 1 else os.environ.get("BASE_URL", "http://localhost:5173")).rstrip("/")

FAILS = []


def check(name, ok, detail=""):
    print(f"  {'PASS' if ok else 'FAIL'}  {name}" + ("" if ok else f"   <- {detail}"))
    if not ok:
        FAILS.append(name)


def section(title):
    print(f"\n{title}")


class Client:
    def __init__(self):
        self.op = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))

    def raw(self, method, path, data=None, headers=None, ctype="application/json"):
        r = urllib.request.Request(BASE + path, data=data, method=method)
        if data is not None:
            r.add_header("content-type", ctype)
        for k, v in (headers or {}).items():
            r.add_header(k, v)
        try:
            resp = self.op.open(r)
            return resp.status, resp.read()
        except urllib.error.HTTPError as e:
            return e.code, e.read()

    def json(self, method, path, body=None, headers=None) -> tuple[int, Any]:
        data = json.dumps(body).encode() if body is not None else None
        status, payload = self.raw(method, path, data, headers)
        try:
            return status, json.loads(payload)
        except Exception:
            return status, None


def new_user(prefix):
    c = Client()
    email = f"{prefix}-{uuid.uuid4().hex[:10]}@example.test"
    c.json("POST", "/api/v1/auth/register", {"email": email, "password": "password123", "name": prefix})
    status, _ = c.json("POST", "/api/v1/auth/login", {"email": email, "password": "password123"})
    assert status == 200, f"login failed for {email}: {status}"
    return c, email


def one_workspace(client, name):
    status, ws = client.json("POST", "/api/v1/workspaces", {"name": name})
    assert status == 201, f"workspace create failed: {status} {ws}"
    return ws["id"], {"x-workspace-id": ws["id"]}


def make_song(client, headers, title, key, tempo):
    status, song = client.json(
        "POST",
        "/api/v1/songs",
        {
            "title": title,
            "artist": "Acceptance",
            "default_key": key,
            "tempo": tempo,
            "chordpro": f"{{title: {title}}}\n{key} line one\n{key} line two",
        },
        headers,
    )
    assert status == 201, f"song create failed: {status} {song}"
    return song


print(f"M2 acceptance against {BASE}")

owner, owner_email = new_user("owner")
WID, H = one_workspace(owner, "M2 Acceptance Band")
SONG1 = make_song(owner, H, "First Song", "G", 72)
SONG2 = make_song(owner, H, "Second Song", "D", 98)

section("SET-1  create / list / detail / update")
status, sl = owner.json(
    "POST",
    "/api/v1/setlists",
    {"title": "Sunday AM", "service_date": "2026-10-25", "service_type": "morning"},
    H,
)
check("POST /setlists -> 201", status == 201, f"{status} {sl}")
SL = sl.get("id") if isinstance(sl, dict) else None
check(
    "header fields echoed",
    isinstance(sl, dict)
    and sl.get("title") == "Sunday AM"
    and sl.get("service_date") == "2026-10-25"
    and sl.get("service_type") == "morning",
    f"{sl}",
)
check("status defaults to draft", isinstance(sl, dict) and sl.get("status") == "draft")
check("no share token by default", isinstance(sl, dict) and sl.get("share_token") is None)

status, res = owner.json("GET", "/api/v1/setlists", None, H)
check(
    "list contains the setlist",
    status == 200 and any(s["id"] == SL for s in res.get("setlists", [])),
    f"{status} {res}",
)

status, detail = owner.json("GET", f"/api/v1/setlists/{SL}", None, H)
check(
    "detail is {setlist, items:[...]}",
    status == 200 and isinstance(detail, dict) and detail.get("items") == [] and detail.get("setlist", {}).get("id") == SL,
    f"{status} {detail}",
)

status, updated = owner.json("PATCH", f"/api/v1/setlists/{SL}", {"title": "Sunday AM (rev)"}, H)
check("PATCH title", status == 200 and updated.get("title") == "Sunday AM (rev)", f"{status} {updated}")

section("SET-2 / SET-5  items")
status, i1 = owner.json(
    "POST", f"/api/v1/setlists/{SL}/items", {"kind": "song", "song_id": SONG1["id"], "duration_seconds": 240}, H
)
check(
    "add song item (position 0, song embedded)",
    status == 201 and i1.get("position") == 0 and i1.get("song", {}).get("title") == "First Song",
    f"{status} {i1}",
)
status, i2 = owner.json("POST", f"/api/v1/setlists/{SL}/items", {"kind": "song", "song_id": SONG2["id"]}, H)
check("second song item -> position 1", status == 201 and i2.get("position") == 1, f"{status} {i2}")
status, note = owner.json(
    "POST", f"/api/v1/setlists/{SL}/items", {"kind": "note", "title": "Prayer", "notes": "Open with prayer"}, H
)
check("non-song item", status == 201 and note.get("position") == 2 and note.get("kind") == "note", f"{status} {note}")
status, _ = owner.json("POST", f"/api/v1/setlists/{SL}/items", {"kind": "note", "notes": "no title"}, H)
check("note without title -> 400", status == 400, f"{status}")

other, _ = new_user("other")
OWS, OH = one_workspace(other, "M2 Other Band")
foreign = make_song(other, OH, "Foreign Song", "C", 100)
status, _ = owner.json("POST", f"/api/v1/setlists/{SL}/items", {"kind": "song", "song_id": foreign["id"]}, H)
check("song from another workspace -> 404", status == 404, f"{status}")

section("SET-3  reorder")
order = [i2["id"], i1["id"], note["id"]]
status, _ = owner.json("PUT", f"/api/v1/setlists/{SL}/items/order", {"item_ids": order}, H)
check("PUT /items/order -> 200", status == 200, f"{status}")
status, detail = owner.json("GET", f"/api/v1/setlists/{SL}", None, H)
items = detail.get("items", []) if isinstance(detail, dict) else []
check(
    "order persists with dense positions",
    [i["id"] for i in items] == order and [i["position"] for i in items] == [0, 1, 2],
    f"{[ (i['id'], i['position']) for i in items ]}",
)
status, _ = owner.json("PUT", f"/api/v1/setlists/{SL}/items/order", {"item_ids": order[:2]}, H)
check("incomplete id set is rejected", status in (400, 409, 422), f"{status}")

section("SET-4  per-item overrides never mutate the song")
status, over = owner.json(
    "PATCH",
    f"/api/v1/setlists/{SL}/items/{i1['id']}",
    {"key_override": "A", "capo_override": 2, "notes": "Capo 2", "duration_seconds": 300},
    H,
)
check(
    "override stored on the item",
    status == 200
    and over.get("key_override") == "A"
    and over.get("capo_override") == 2
    and over.get("duration_seconds") == 300,
    f"{status} {over}",
)
status, song = owner.json("GET", f"/api/v1/songs/{SONG1['id']}", None, H)
check(
    "song is unchanged by overrides",
    status == 200 and song.get("default_key") == "G" and song.get("tempo") == 72,
    f"{status} {song.get('default_key')}",
)
status, cleared = owner.json("PATCH", f"/api/v1/setlists/{SL}/items/{i1['id']}", {"key_override": None}, H)
check("override can be cleared (reset to library)", status == 200 and cleared.get("key_override") is None, f"{status} {cleared}")

section("SET-7  duplicate + template")
status, dup = owner.json("POST", f"/api/v1/setlists/{SL}/duplicate", None, H)
DUP = dup.get("id") if isinstance(dup, dict) else None
check("duplicate -> 201 new setlist", status == 201 and DUP and DUP != SL, f"{status} {dup}")
status, dd = owner.json("GET", f"/api/v1/setlists/{DUP}", None, H)
dup_items = dd.get("items", []) if isinstance(dd, dict) else []
check(
    "duplicate copies items + overrides",
    status == 200 and len(dup_items) == 3 and any(i.get("notes") == "Capo 2" for i in dup_items),
    f"{status} {len(dup_items)}",
)
check(
    "duplicate resets status/template/share",
    dd.get("setlist", {}).get("status") == "draft"
    and dd.get("setlist", {}).get("is_template") is False
    and dd.get("setlist", {}).get("share_token") is None,
    f"{dd.get('setlist')}",
)
status, tpl = owner.json("PATCH", f"/api/v1/setlists/{SL}", {"is_template": True}, H)
check("save as template", status == 200 and tpl.get("is_template") is True, f"{status} {tpl}")

section("SET-8  share link")
status, share = owner.json("POST", f"/api/v1/setlists/{SL}/share", None, H)
TOKEN = share.get("share_token") if isinstance(share, dict) else None
check("share -> 200 with token >= 32 chars", status == 200 and TOKEN and len(TOKEN) >= 32, f"{status} {share}")
status, again = owner.json("POST", f"/api/v1/setlists/{SL}/share", None, H)
check("share is idempotent", status == 200 and again.get("share_token") == TOKEN, f"{status} {again}")

anon = Client()
status, pub = anon.json("GET", f"/api/v1/shared/setlists/{TOKEN}")
check("public share works with no cookie and no workspace header", status == 200, f"{status} {pub}")
pub_items = pub.get("items", []) if isinstance(pub, dict) else []
check(
    "public payload has items + embedded chart",
    len(pub_items) == 3 and any(i.get("song", {}).get("chordpro") for i in pub_items),
    f"{len(pub_items)}",
)
blob = json.dumps(pub).lower()
check("public payload leaks no emails/memberships", "@example.test" not in blob and "memberships" not in blob)
check("public payload exposes no member/asset data", "created_by" not in blob and '"assets"' not in blob)

status, _ = owner.json("DELETE", f"/api/v1/setlists/{SL}/share", None, H)
check("revoke -> 204", status == 204, f"{status}")
status, _ = anon.json("GET", f"/api/v1/shared/setlists/{TOKEN}")
check("revoked token -> 404", status == 404, f"{status}")

section("SET-11  status + archived lock")
status, _ = owner.json("PATCH", f"/api/v1/setlists/{SL}", {"status": "ready"}, H)
check("status -> ready", status == 200, f"{status}")
status, _ = owner.json("POST", f"/api/v1/setlists/{SL}/items", {"kind": "note", "title": "Announcements"}, H)
check("ready setlist still accepts items", status == 201, f"{status}")
status, _ = owner.json("PATCH", f"/api/v1/setlists/{SL}", {"status": "archived"}, H)
check("status -> archived", status == 200, f"{status}")
status, _ = owner.json("POST", f"/api/v1/setlists/{SL}/items", {"kind": "note", "title": "Late"}, H)
check("archived blocks item creation (409)", status == 409, f"{status}")
status, _ = owner.json("POST", f"/api/v1/setlists/{SL}/share", None, H)
check("archived blocks share creation (409)", status == 409, f"{status}")
status, _ = owner.json("GET", f"/api/v1/setlists/{SL}", None, H)
check("archived is still readable", status == 200, f"{status}")

section("validation + tenant isolation")
status, _ = owner.json("POST", "/api/v1/setlists", {}, H)
check("missing title -> 422", status == 422, f"{status}")
status, _ = owner.json("GET", "/api/v1/setlists", None, {})
check("missing X-Workspace-Id -> 422", status == 422, f"{status}")
status, _ = anon.json("GET", "/api/v1/setlists", None, H)
check("unauthenticated -> 401", status == 401, f"{status}")
status, _ = other.json("GET", f"/api/v1/setlists/{SL}", None, OH)
check("cross-tenant read -> 404", status == 404, f"{status}")
status, _ = other.json("PATCH", f"/api/v1/setlists/{SL}", {"title": "sneak"}, OH)
check("cross-tenant write -> 404", status == 404, f"{status}")

section("role gating (viewer)")
viewer_email = f"viewer-{uuid.uuid4().hex[:10]}@example.test"
status, inv = owner.json("POST", f"/api/v1/workspaces/{WID}/invites", {"email": viewer_email, "role": "viewer"}, H)
check("owner invites a viewer", status == 201 and inv.get("token"), f"{status}")
viewer = Client()
viewer.json("POST", "/api/v1/auth/register", {"email": viewer_email, "password": "password123", "name": "viewer"})
viewer.json("POST", "/api/v1/auth/login", {"email": viewer_email, "password": "password123"})
status, _ = viewer.json("POST", f"/api/v1/invites/{inv['token']}/accept")
check("viewer accepts the invite", status == 201, f"{status}")
status, _ = viewer.json("GET", "/api/v1/setlists", None, H)
check("viewer can read setlists", status == 200, f"{status}")
status, _ = viewer.json("POST", "/api/v1/setlists", {"title": "Nope"}, H)
check("viewer cannot create a setlist (403)", status == 403, f"{status}")

section("song delete is restricted while referenced")
throwaway = make_song(owner, H, "Throwaway", "E", 120)
status, tw = owner.json("POST", f"/api/v1/setlists/{DUP}/items", {"kind": "song", "song_id": throwaway["id"]}, H)
status, _ = owner.json("DELETE", f"/api/v1/songs/{throwaway['id']}", None, H)
check("deleting a referenced song -> 409", status == 409, f"{status}")
owner.json("DELETE", f"/api/v1/setlists/{DUP}/items/{tw['id']}", None, H)
status, _ = owner.json("DELETE", f"/api/v1/songs/{throwaway['id']}", None, H)
check("deleting it after removing the item -> 204", status == 204, f"{status}")

section("delete setlists")
status, _ = owner.json("DELETE", f"/api/v1/setlists/{DUP}", None, H)
check("delete duplicate -> 204", status == 204, f"{status}")
status, _ = owner.json("DELETE", f"/api/v1/setlists/{SL}", None, H)
check("delete setlist -> 204", status == 204, f"{status}")
status, _ = owner.json("GET", f"/api/v1/setlists/{SL}", None, H)
check("deleted setlist -> 404", status == 404, f"{status}")

print("\n" + "=" * 60)
if FAILS:
    print(f"RESULT: FAIL — {len(FAILS)} check(s) failed: {', '.join(FAILS)}")
    sys.exit(1)
print("RESULT: PASS — all M2 acceptance checks passed")
print(f"(owner account: {owner_email})")
