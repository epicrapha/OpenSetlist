-- M0 baseline schema.
--
-- Tenant-scoped tables (workspaces, memberships, invites) carry the workspace
-- id and get Row Level Security in 0002_rls.sql.
-- sessions live in the tower_sessions schema, managed by tower-sessions-sqlx-store.

CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE users (
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email         citext NOT NULL UNIQUE,
    name          text NOT NULL,
    password_hash text NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE workspaces (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name       text NOT NULL,
    settings   jsonb NOT NULL DEFAULT '{}'::jsonb,
    plan       text NOT NULL DEFAULT 'free',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TYPE membership_role AS ENUM ('owner', 'admin', 'leader', 'member', 'viewer');

CREATE TABLE memberships (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    role         membership_role NOT NULL DEFAULT 'member',
    created_at   timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, workspace_id)
);

CREATE INDEX memberships_user_id_idx ON memberships (user_id);
CREATE INDEX memberships_workspace_id_idx ON memberships (workspace_id);

CREATE TABLE invites (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    email        citext NOT NULL,
    role         membership_role NOT NULL DEFAULT 'member',
    token        text NOT NULL UNIQUE,
    expires_at   timestamptz NOT NULL,
    accepted_at  timestamptz,
    created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX invites_workspace_id_idx ON invites (workspace_id);
