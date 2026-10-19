-- M2 setlists.
--
-- Tenant isolation mirrors 0002/0003. Public sharing uses a permissive,
-- token-scoped SELECT policy keyed on `app.current_share_token` (the same shape
-- as `invites_by_token` in 0004): no tenant is set for public reads, so only
-- rows reachable from the matching share token become visible.

CREATE TABLE setlists (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title        text NOT NULL,
    service_date date,
    service_type text,
    status       text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'ready', 'archived')),
    is_template  boolean NOT NULL DEFAULT false,
    share_token  text,
    created_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX setlists_workspace_updated_idx ON setlists (workspace_id, updated_at desc);
CREATE UNIQUE INDEX setlists_share_token_unique_idx
    ON setlists (share_token) WHERE share_token IS NOT NULL;

CREATE TABLE setlist_items (
    id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id     uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    setlist_id       uuid NOT NULL REFERENCES setlists(id) ON DELETE CASCADE,
    position         integer NOT NULL,
    kind             text NOT NULL CHECK (kind IN ('song', 'note')),
    -- RESTRICT: deleting a song must never silently drop it from a setlist.
    song_id          uuid REFERENCES songs(id) ON DELETE RESTRICT,
    title            text,
    notes            text,
    key_override     text,
    capo_override    smallint,
    duration_seconds integer,
    created_at       timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX setlist_items_setlist_position_idx ON setlist_items (setlist_id, position);

GRANT SELECT, INSERT, UPDATE, DELETE ON setlists, setlist_items TO opensetlist_app;

ALTER TABLE setlists ENABLE ROW LEVEL SECURITY;
ALTER TABLE setlists FORCE ROW LEVEL SECURITY;
ALTER TABLE setlist_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE setlist_items FORCE ROW LEVEL SECURITY;

CREATE POLICY setlists_isolation ON setlists
    USING (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid);

CREATE POLICY setlist_items_isolation ON setlist_items
    USING (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid);

-- Public share: exactly the setlist whose token was presented.
CREATE POLICY setlists_share_read ON setlists
    FOR SELECT
    USING (
        share_token IS NOT NULL
        AND share_token = nullif(current_setting('app.current_share_token', true), '')
    );

-- Public share: items belonging to the shared setlist only.
CREATE POLICY setlist_items_share_read ON setlist_items
    FOR SELECT
    USING (
        setlist_id IN (
            SELECT s.id FROM setlists s
            WHERE s.share_token IS NOT NULL
              AND s.share_token = nullif(current_setting('app.current_share_token', true), '')
        )
    );

-- Public share: embedded song fields, but only songs referenced by the shared
-- setlist. Nothing else in the songs table (or any other workspace) is exposed.
CREATE POLICY songs_share_read ON songs
    FOR SELECT
    USING (
        EXISTS (
            SELECT 1
            FROM setlist_items si
            JOIN setlists s ON s.id = si.setlist_id
            WHERE si.song_id = songs.id
              AND s.share_token IS NOT NULL
              AND s.share_token = nullif(current_setting('app.current_share_token', true), '')
        )
    );
