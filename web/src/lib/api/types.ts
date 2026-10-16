/** Domain types for the M0 API contract (PLAN.md §3). */

export type Role = 'owner' | 'admin' | 'leader' | 'member' | 'viewer';

export interface User {
	id: string;
	email: string;
	name: string;
	created_at?: string;
}

export interface Membership {
	id: string;
	user_id: string;
	workspace_id: string;
	role: Role;
}

export interface Workspace {
	id: string;
	name: string;
	plan?: string;
	settings?: unknown;
	created_at?: string;
}

/** `GET /api/v1/auth/me` — current user + memberships. */
export interface Me {
	user: User;
	memberships: Membership[];
}

/** `POST /api/v1/workspaces` — workspace + owner membership. */
export interface WorkspaceCreated {
	workspace: Workspace;
	membership: Membership;
}
