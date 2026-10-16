#!/usr/bin/env bash
# M0 acceptance: exercises the whole stack THROUGH nginx (the /api/ proxy),
# not the server directly. Requires the compose stack to be up:
#   docker compose up -d
# Usage: scripts/m0-acceptance.sh   (BASE=http://localhost:3000 to override)
set -uo pipefail

BASE="${BASE:-http://localhost:3000}"
RUN_ID="$(date +%s)-$RANDOM"
PASSWORD="password123"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

FAILURES=0
BODY="$TMP/body.json"

check() { # description expected actual
	local desc="$1" expected="$2" actual="$3"
	if [ "$expected" = "$actual" ]; then
		printf 'PASS  %-52s expected=%-3s got=%s\n' "$desc" "$expected" "$actual"
	else
		printf 'FAIL  %-52s expected=%-3s got=%s\n' "$desc" "$expected" "$actual"
		FAILURES=$((FAILURES + 1))
	fi
}

# req METHOD PATH [cookie-jar] [json-body]  -> prints HTTP status, body in $BODY
req() {
	local method="$1" path="$2" jar="${3:-}" body="${4:-}"
	local -a args=(-s -o "$BODY" -w '%{http_code}' -X "$method" "$BASE$path"
		-H 'content-type: application/json')
	[ -n "$jar" ] && args+=(-b "$jar" -c "$jar")
	[ -n "$body" ] && args+=(-d "$body")
	curl "${args[@]}"
}

jassert() { # description jq-filter file
	if jq -e "$2" "$3" >/dev/null 2>&1; then
		printf 'PASS  %s\n' "$1"
	else
		printf 'FAIL  %s (jq: %s)\n' "$1" "$2"
		FAILURES=$((FAILURES + 1))
	fi
}

echo "== M0 acceptance against $BASE (run $RUN_ID) =="

echo "-- health --"
code="$(req GET /health)"
check 'GET /health' 200 "$code"
jassert '/health returns {"status":"ok"}' '.status == "ok"' "$BODY"

echo "-- owner register / login / me --"
OWNER_EMAIL="owner-$RUN_ID@example.test"
code="$(req POST /api/v1/auth/register '' \
	"{\"email\":\"$OWNER_EMAIL\",\"password\":\"$PASSWORD\",\"name\":\"Owner\"}")"
check 'POST /api/v1/auth/register (owner)' 201 "$code"

OWNER_JAR="$TMP/owner.jar"
code="$(req POST /api/v1/auth/login "$OWNER_JAR" \
	"{\"email\":\"$OWNER_EMAIL\",\"password\":\"$PASSWORD\"}")"
check 'POST /api/v1/auth/login (owner)' 200 "$code"
if grep -q 'opensetlist.sid' "$OWNER_JAR" 2>/dev/null; then
	printf 'PASS  %s\n' 'login set an opensetlist.sid session cookie'
else
	printf 'FAIL  %s\n' 'login set an opensetlist.sid session cookie'
	FAILURES=$((FAILURES + 1))
fi

code="$(req GET /api/v1/auth/me "$OWNER_JAR")"
check 'GET /api/v1/auth/me (owner, authenticated)' 200 "$code"

echo "-- workspace create / read / invite --"
code="$(req POST /api/v1/workspaces "$OWNER_JAR" '{"name":"Acceptance Band A"}')"
check 'POST /api/v1/workspaces (owner)' 201 "$code"
A_ID="$(jq -r '.id // empty' "$BODY")"

code="$(req GET "/api/v1/workspaces/$A_ID" "$OWNER_JAR")"
check 'GET /api/v1/workspaces/{A} (owner/member)' 200 "$code"

code="$(req POST "/api/v1/workspaces/$A_ID/invites" "$OWNER_JAR" \
	'{"email":"invitee@example.test","role":"member"}')"
check 'POST /api/v1/workspaces/{A}/invites (owner)' 201 "$code"

echo "-- M0-A2: second tenant must not read the first --"
USERB_EMAIL="userb-$RUN_ID@example.test"
code="$(req POST /api/v1/auth/register '' \
	"{\"email\":\"$USERB_EMAIL\",\"password\":\"$PASSWORD\",\"name\":\"UserB\"}")"
check 'POST /api/v1/auth/register (user B)' 201 "$code"

USERB_JAR="$TMP/userb.jar"
code="$(req POST /api/v1/auth/login "$USERB_JAR" \
	"{\"email\":\"$USERB_EMAIL\",\"password\":\"$PASSWORD\"}")"
check 'POST /api/v1/auth/login (user B)' 200 "$code"

code="$(req POST /api/v1/workspaces "$USERB_JAR" '{"name":"Acceptance Band B"}')"
check 'POST /api/v1/workspaces (user B)' 201 "$code"
B_ID="$(jq -r '.id // empty' "$BODY")"

code="$(req GET "/api/v1/workspaces/$A_ID" "$USERB_JAR")"
check "GET /api/v1/workspaces/{A} as user B (cross-tenant)" 404 "$code"
jassert 'cross-tenant denial uses the typed not_found error' '.error.code == "not_found"' "$BODY"
jassert 'cross-tenant denial leaks no workspace row' '
	(has("id") | not) and (has("name") | not) and (has("settings") | not)' "$BODY"

code="$(req GET /api/v1/auth/me "$USERB_JAR")"
check 'GET /api/v1/auth/me (user B)' 200 "$code"
if jq -e --arg a "$A_ID" '[.memberships[].workspace_id] | index($a) == null' "$BODY" >/dev/null 2>&1; then
	printf "PASS  %s\n" "user B's memberships exclude tenant A (no row leak)"
else
	printf "FAIL  %s\n" "user B's memberships exclude tenant A (no row leak)"
	FAILURES=$((FAILURES + 1))
fi

echo "-- unauthenticated --"
code="$(req GET /api/v1/auth/me)"
check 'GET /api/v1/auth/me (anonymous)' 401 "$code"
jassert 'anonymous denial is a typed unauthorized error' '.error.code == "unauthorized"' "$BODY"

echo
if [ "$FAILURES" -eq 0 ]; then
	echo "RESULT: PASS (all checks)"
	exit 0
else
	echo "RESULT: FAIL ($FAILURES check(s) failed)"
	exit 1
fi
