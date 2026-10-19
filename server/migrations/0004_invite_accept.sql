-- Invite acceptance needs to resolve a single invite by its token *before* the
-- workspace (tenant) is known, so the ordinary `invites_isolation` policy
-- (workspace_id = current_tenant_id) cannot see the row yet.
--
-- Rather than bypassing RLS, add a permissive SELECT policy keyed on a second
-- GUC, `app.current_invite_token`, set per transaction by the application. The
-- token is the secret and is single-use; the row is only visible when the
-- presented token matches. Permissive policies are OR'd, so normal tenant
-- reads via `invites_isolation` are unaffected.

CREATE POLICY invites_by_token ON invites
    FOR SELECT
    USING (token = nullif(current_setting('app.current_invite_token', true), ''));
