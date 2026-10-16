import { ApiError, getWorkspace, me } from './api';
import type { Role, User } from './api';

export interface WorkspaceSummary {
	id: string;
	name: string;
	role: Role;
}

export type SessionStatus = 'idle' | 'loading' | 'ready' | 'error';

const ACTIVE_KEY = 'opensetlist:workspace';

function storedActiveId(): string | null {
	try {
		return localStorage.getItem(ACTIVE_KEY);
	} catch {
		return null;
	}
}

// ponytail: one app-wide session holder — M0 has a single authed area. Split into
// context if a second authed shell ever appears.
export const session = $state({
	status: 'idle' as SessionStatus,
	error: '',
	user: null as User | null,
	workspaces: [] as WorkspaceSummary[],
	activeWorkspaceId: storedActiveId()
});

export function setActiveWorkspace(id: string | null): void {
	session.activeWorkspaceId = id;
	try {
		if (id) localStorage.setItem(ACTIVE_KEY, id);
		else localStorage.removeItem(ACTIVE_KEY);
	} catch {
		// storage unavailable: selection stays in memory for this tab.
	}
}

export function resetSession(): void {
	session.status = 'idle';
	session.error = '';
	session.user = null;
	session.workspaces = [];
}

/** Load the current user and their workspaces once. `force` refetches. */
export async function loadSession(force = false): Promise<void> {
	if (!force && (session.status === 'loading' || session.status === 'ready')) return;
	session.status = 'loading';
	session.error = '';
	try {
		const data = await me();
		session.user = data.user;
		session.workspaces = await Promise.all(
			data.memberships.map(async (membership) => {
				const name = await getWorkspace(membership.workspace_id)
					.then((workspace) => workspace.name)
					.catch(() => membership.workspace_id);
				return { id: membership.workspace_id, name, role: membership.role };
			})
		);
		if (!session.workspaces.some((workspace) => workspace.id === session.activeWorkspaceId)) {
			setActiveWorkspace(session.workspaces[0]?.id ?? null);
		}
		session.status = 'ready';
	} catch (cause) {
		session.status = 'error';
		session.error = cause instanceof ApiError ? cause.message : 'Could not load your account.';
	}
}
