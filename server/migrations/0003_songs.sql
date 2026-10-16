-- M1 song library.
--
-- Every table carries workspace_id so RLS can scope it directly (PLAN §4) and
-- indexes can lead with the tenant. Arrangements/assets denormalise it from
-- their owning song; the application inserts it from the resolved workspace
-- scope, and RLS is the backstop.

CREATE TABLE songs (
    id             uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id   uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title          text NOT NULL,
    artist         text,
    default_key    text,
    tempo          integer,
    time_signature text NOT NULL DEFAULT '4/4',
    tags           text[] NOT NULL DEFAULT '{}',
    ccli_number    text,
    copyright      text,
    chordpro       text NOT NULL DEFAULT '',
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX songs_workspace_title_idx ON songs (workspace_id, lower(title));
CREATE INDEX songs_workspace_artist_idx ON songs (workspace_id, artist);
CREATE INDEX songs_workspace_updated_idx ON songs (workspace_id, updated_at);
CREATE INDEX songs_tags_idx ON songs USING gin (tags);

CREATE TABLE song_arrangements (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    song_id      uuid NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    name         text NOT NULL,
    key          text,
    chordpro     text NOT NULL DEFAULT '',
    created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX song_arrangements_song_idx ON song_arrangements (song_id);

CREATE TABLE song_assets (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    song_id      uuid NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    kind         text NOT NULL CHECK (kind IN ('link', 'pdf')),
    -- Reference link (kind='link'); null for uploads.
    url          text,
    label        text,
    -- Upload metadata (kind='pdf'); null for links.
    filename     text,
    size_bytes   bigint,
    content_type text,
    -- Random on-disk object name; never the client filename. Internal only.
    object_name  text,
    created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX song_assets_song_idx ON song_assets (song_id);

GRANT SELECT, INSERT, UPDATE, DELETE ON songs, song_arrangements, song_assets TO opensetlist_app;

ALTER TABLE songs ENABLE ROW LEVEL SECURITY;
ALTER TABLE songs FORCE ROW LEVEL SECURITY;
ALTER TABLE song_arrangements ENABLE ROW LEVEL SECURITY;
ALTER TABLE song_arrangements FORCE ROW LEVEL SECURITY;
ALTER TABLE song_assets ENABLE ROW LEVEL SECURITY;
ALTER TABLE song_assets FORCE ROW LEVEL SECURITY;

CREATE POLICY songs_isolation ON songs
    USING (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid);

CREATE POLICY song_arrangements_isolation ON song_arrangements
    USING (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid);

CREATE POLICY song_assets_isolation ON song_assets
    USING (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid);
