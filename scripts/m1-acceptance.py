#!/usr/bin/env python3
"""M1 acceptance — song library, end-to-end over HTTP.

Covers PRD SONG-1..7, SONG-9, SONG-11 plus auth/tenant isolation and the
asset-list contract (PLAN.md §3.1).

Usage:
    python3 scripts/m1-acceptance.py [BASE_URL]
    BASE_URL defaults to http://localhost:5173 (dev) — use :3000 for compose.

Role gating (leader+ for writes) is NOT exercised here: there is no accept-invite
endpoint yet, so a non-leader member cannot be created over HTTP. It is proven by
the executable integration test `writes_require_leader_role` in server/tests/songs.rs.
"""
import http.cookiejar
import json
import os
import sys
import urllib.error
import urllib.request
import uuid

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

    def json(self, method, path, body=None, headers=None):
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


def multipart(field, filename, content, ctype):
    boundary = uuid.uuid4().hex
    body = (
        f'--{boundary}\r\nContent-Disposition: form-data; name="{field}"; filename="{filename}"\r\n'
        f"Content-Type: {ctype}\r\n\r\n"
    ).encode() + content + f"\r\n--{boundary}--\r\n".encode()
    return body, f"multipart/form-data; boundary={boundary}"


PDF = (
    b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n"
    b"2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n"
    b"3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 200]>>endobj\n"
    b"trailer<</Root 1 0 R>>\n%%EOF\n"
)

print(f"M1 acceptance against {BASE}")

owner, owner_email = new_user("owner")
status, ws = owner.json("POST", "/api/v1/workspaces", {"name": "Acceptance Band"})
check("create workspace", status == 201 and ws and "id" in ws, f"{status} {ws}")
WID = ws["id"]
H = {"x-workspace-id": WID}

section("SONG-1  create with full metadata")
CHORDPRO = (
    "{title: 10,000 Reasons}\n{key: G}\n\n"
    "[G]Bless the Lord, O my [D]soul\n[Em]O my soul [C]worship His holy name"
)
SONG = {
    "title": "10,000 Reasons",
    "artist": "Matt Redman",
    "default_key": "G",
    "tempo": 73,
    "time_signature": "4/4",
    "tags": ["worship", "upbeat"],
    "ccli_number": "6016351",
    "copyright": "2011 Thankyou Music",
    "chordpro": CHORDPRO,
}
status, song = owner.json("POST", "/api/v1/songs", SONG, H)
ok = status == 201 and all(song.get(k) == SONG[k] for k in ("title", "artist", "default_key", "tempo", "time_signature", "tags", "ccli_number", "copyright"))
check("POST /songs echoes every field", ok, f"{status} {json.dumps(song)[:220]}")
SID = song["id"]

status, got = owner.json("GET", f"/api/v1/songs/{SID}", None, H)
check("GET /songs/{id}", status == 200 and got["chordpro"] == CHORDPRO, f"{status}")

status, updated = owner.json("PATCH", f"/api/v1/songs/{SID}", {"tempo": 76, "tags": ["worship"]}, H)
check("PATCH /songs/{id} partial update", status == 200 and updated["tempo"] == 76 and updated["tags"] == ["worship"], f"{status}")

section("SONG-9  search / filter / sort / paginate")
for t, k, tag in [("Amazing Grace", "D", "hymn"), ("Great Are You Lord", "A", "worship")]:
    owner.json("POST", "/api/v1/songs", {"title": t, "default_key": k, "tags": [tag], "chordpro": f"{{title: {t}}}\n[{k}]line"}, H)

status, res = owner.json("GET", "/api/v1/songs?q=Amazing", None, H)
check("q= filters by title", status == 200 and res["total"] == 1 and res["songs"][0]["title"] == "Amazing Grace", f"{status} {res}")
status, res = owner.json("GET", "/api/v1/songs?key=A", None, H)
check("key= filter", status == 200 and res["total"] == 1 and res["songs"][0]["title"] == "Great Are You Lord", f"{status} {res}")
status, res = owner.json("GET", "/api/v1/songs?tag=hymn", None, H)
check("tag= filter", status == 200 and res["total"] == 1, f"{status} {res}")
status, res = owner.json("GET", "/api/v1/songs?sort=title", None, H)
titles = [s["title"] for s in res["songs"]]
check("sort=title", status == 200 and titles == sorted(titles), f"{status} {titles}")
status, res = owner.json("GET", "/api/v1/songs?limit=2&offset=0", None, H)
check("limit/offset + total", status == 200 and len(res["songs"]) == 2 and res["total"] == 3, f"{status} {res.get('total')}")
status, res = owner.json("GET", "/api/v1/songs?q=zzz-no-match", None, H)
check("no-match search is empty, not an error", status == 200 and res["total"] == 0, f"{status}")

section("SONG-11  export")
status, body = owner.raw("GET", f"/api/v1/songs/{SID}/export?format=chordpro", None, H)
check("export chordpro", status == 200 and b"[G]Bless the Lord" in body, f"{status}")
status, body = owner.raw("GET", f"/api/v1/songs/{SID}/export?format=text", None, H)
check("export text strips chord tokens", status == 200 and b"Bless the Lord" in body and b"[G]" not in body, f"{status} {body[:80]!r}")

section("duplicate")
status, dup = owner.json("POST", f"/api/v1/songs/{SID}/duplicate", None, H)
check("POST /songs/{id}/duplicate", status == 201 and dup["id"] != SID and dup["chordpro"] == CHORDPRO, f"{status}")

section("SONG-6  reference links")
status, _ = owner.json("POST", f"/api/v1/songs/{SID}/assets", {"kind": "link", "url": "https://www.youtube.com/watch?v=abc", "label": "YouTube"}, H)
check("create link asset", status == 201, f"{status}")
status, assets = owner.json("GET", f"/api/v1/songs/{SID}/assets", None, H)
links = [a for a in assets["assets"] if a["kind"] == "link"]
check("GET /songs/{id}/assets lists the link", status == 200 and len(links) == 1 and links[0]["label"] == "YouTube", f"{status} {assets}")
check("asset never leaks object_name", all("object_name" not in a for a in assets["assets"]))

section("SONG-7  PDF upload")
body, ctype = multipart("file", "chart.pdf", PDF, "application/pdf")
status, _ = owner.raw("POST", f"/api/v1/songs/{SID}/assets/pdf", body, H, ctype)
check("upload application/pdf", status == 201, f"{status}")
status, assets = owner.json("GET", f"/api/v1/songs/{SID}/assets", None, H)
pdfs = [a for a in assets["assets"] if a["kind"] == "pdf"]
check("uploaded PDF appears in the asset list", status == 200 and len(pdfs) == 1 and pdfs[0]["content_type"] == "application/pdf", f"{status} {assets}")
PDF_ID = pdfs[0]["id"] if pdfs else None
status, blob = owner.raw("GET", f"/api/v1/assets/{PDF_ID}/file", None, H)
check("download PDF returns the bytes", status == 200 and blob == PDF, f"{status} {len(blob or b'')}B")

body, ctype = multipart("file", "notes.txt", b"not a pdf", "text/plain")
status, _ = owner.raw("POST", f"/api/v1/songs/{SID}/assets/pdf", body, H, ctype)
check("non-PDF is rejected (415)", status == 415, f"{status}")

big = PDF + b"0" * (10 * 1024 * 1024 + 2048)
body, ctype = multipart("file", "big.pdf", big, "application/pdf")
status, _ = owner.raw("POST", f"/api/v1/songs/{SID}/assets/pdf", body, H, ctype)
check("oversize PDF is rejected (413)", status == 413, f"{status}")

section("SONG-8  arrangements")
status, arr = owner.json("POST", f"/api/v1/songs/{SID}/arrangements", {"name": "Sunday", "key": "A", "chordpro": "[A]line"}, H)
check("create arrangement", status == 201, f"{status}")
ARR = arr["id"] if isinstance(arr, dict) else None
status, arrs = owner.json("GET", f"/api/v1/songs/{SID}/arrangements", None, H)
count = len(arrs["arrangements"]) if isinstance(arrs, dict) else len(arrs or [])
check("list arrangements", status == 200 and count == 1, f"{status} {arrs}")
status, _ = owner.json("PATCH", f"/api/v1/arrangements/{ARR}", {"name": "Sunday AM"}, H)
check("patch arrangement", status == 200, f"{status}")
status, _ = owner.json("DELETE", f"/api/v1/arrangements/{ARR}", None, H)
check("delete arrangement", status == 204, f"{status}")

section("auth + tenant isolation")
status, _ = owner.json("GET", "/api/v1/songs", None, {})
check("missing X-Workspace-Id is a validation error (422)", status == 422, f"{status}")
anon = Client()
status, _ = anon.json("GET", "/api/v1/songs", None, H)
check("unauthenticated read is 401", status == 401, f"{status}")

other, _ = new_user("other")
status, ows = other.json("POST", "/api/v1/workspaces", {"name": "Other Band"})
OWS = ows["id"]
OH = {"x-workspace-id": OWS}
status, _ = other.json("GET", f"/api/v1/songs/{SID}", None, OH)
check("cross-tenant song read is 404", status == 404, f"{status}")
status, _ = other.json("GET", f"/api/v1/songs/{SID}/assets", None, OH)
check("cross-tenant asset list is 404", status == 404, f"{status}")
status, _ = other.json("POST", "/api/v1/songs", {"title": "sneaky", "chordpro": "x"}, OH)
check("write into own workspace is allowed (sanity)", status == 201, f"{status}")

section("delete")
status, _ = owner.json("DELETE", f"/api/v1/songs/{SID}", None, H)
check("delete song", status == 204, f"{status}")
status, _ = owner.json("GET", f"/api/v1/songs/{SID}", None, H)
check("deleted song is 404", status == 404, f"{status}")

print("\n" + "=" * 60)
if FAILS:
    print(f"RESULT: FAIL — {len(FAILS)} check(s) failed: {', '.join(FAILS)}")
    sys.exit(1)
print("RESULT: PASS — all M1 acceptance checks passed")
print(f"(owner account: {owner_email})")
