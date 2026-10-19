import { ApiError } from './api';
import type { Invite } from './api';

export type InviteStatus = 'pending' | 'accepted' | 'expired';

/** Pending is the only state a link can still be used in. */
export function inviteStatus(invite: Invite, now: number = Date.now()): InviteStatus {
	if (invite.accepted_at) return 'accepted';
	return new Date(invite.expires_at).getTime() <= now ? 'expired' : 'pending';
}

export type AcceptErrorKind =
	'unknown' | 'expired' | 'accepted' | 'member' | 'wrong-email' | 'signed-out' | 'other';

/**
 * The server uses status 409 for all "can't accept" cases and only distinguishes
 * them in the message (PLAN.md §3.2), so map that text to a distinct outcome.
 */
export function acceptErrorKind(cause: unknown): AcceptErrorKind {
	if (!(cause instanceof ApiError)) return 'other';
	if (cause.status === 401) return 'signed-out';
	if (cause.status === 403) return 'wrong-email';
	if (cause.status === 404) return 'unknown';
	if (cause.status === 409) {
		const message = cause.message.toLowerCase();
		if (message.includes('expired')) return 'expired';
		if (message.includes('already a member')) return 'member';
		if (message.includes('accepted')) return 'accepted';
	}
	return 'other';
}

export function invitePath(token: string): string {
	return `/invite/${encodeURIComponent(token)}`;
}

export function inviteLink(origin: string, token: string): string {
	return `${origin.replace(/\/+$/, '')}${invitePath(token)}`;
}
