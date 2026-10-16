-- Tenant isolation via PostgreSQL Row Level Security.
--
-- Policies read `app.current_tenant_id` (the active workspace) and
-- `app.current_user_id` (the authenticated user). Both are set per transaction
-- by the application (`AppState::scoped`). `nullif(..., '')` is required
-- because a custom GUC that was set once and then rolled back reads back as an
-- empty string rather than NULL; without it the `::uuid` cast would error.
--
-- IMPORTANT: superusers and roles with BYPASSRLS ignore RLS entirely, and table
-- owners bypass it unless FORCE is set. The application therefore runs tenant
-- queries under the non-superuser role `opensetlist_app` (via SET LOCAL ROLE),
-- which is what makes these policies enforce. FORCE is kept as defence in depth.

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'opensetlist_app') THEN
        CREATE ROLE opensetlist_app NOLOGIN;
    END IF;
END
$$;

-- Allow the connection user to assume the app role. Works whether the user is a
-- superuser (dev) or a non-superuser database owner (prod).
DO $$
BEGIN
    EXECUTE format('GRANT opensetlist_app TO %I', current_user);
EXCEPTION WHEN OTHERS THEN
    NULL;
END
$$;

GRANT USAGE ON SCHEMA public TO opensetlist_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON users, workspaces, memberships, invites TO opensetlist_app;

ALTER TABLE workspaces ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspaces FORCE ROW LEVEL SECURITY;
ALTER TABLE memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE memberships FORCE ROW LEVEL SECURITY;
ALTER TABLE invites ENABLE ROW LEVEL SECURITY;
ALTER TABLE invites FORCE ROW LEVEL SECURITY;

-- A workspace is visible only when it is the active tenant.
CREATE POLICY workspaces_isolation ON workspaces
    USING (id = nullif(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (id = nullif(current_setting('app.current_tenant_id', true), '')::uuid);

-- Memberships are visible in the active tenant, or when they belong to the
-- current user (so /auth/me can list them without a tenant).
CREATE POLICY memberships_isolation ON memberships
    USING (
        workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid
        OR user_id = nullif(current_setting('app.current_user_id', true), '')::uuid
    )
    WITH CHECK (
        workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid
        OR user_id = nullif(current_setting('app.current_user_id', true), '')::uuid
    );

-- Invites belong to exactly one tenant.
CREATE POLICY invites_isolation ON invites
    USING (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (workspace_id = nullif(current_setting('app.current_tenant_id', true), '')::uuid);
