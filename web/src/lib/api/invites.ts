import { request } from './client';
import type { AcceptInviteResult, Invite, InviteRole } from './types';

export const listInvites = async (workspaceId: string): Promise<Invite[]> =>
	(
		await request<{ invites: Invite[] }>(`/workspaces/${encodeURIComponent(workspaceId)}/invites`, {
			workspaceId
		})
	).invites;

export const createInvite = (
	workspaceId: string,
	input: { email: string; role: InviteRole }
): Promise<Invite> =>
	request<Invite>(`/workspaces/${encodeURIComponent(workspaceId)}/invites`, {
		method: 'POST',
		body: input,
		workspaceId
	});

/** Not workspace-scoped: the caller may not belong to any workspace yet. */
export const acceptInvite = (token: string): Promise<AcceptInviteResult> =>
	request<AcceptInviteResult>(`/invites/${encodeURIComponent(token)}/accept`, { method: 'POST' });
