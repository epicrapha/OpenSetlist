import { describe, expect, it } from 'vitest';
import { ApiError } from './api';
import { acceptErrorKind, inviteStatus } from './invite';
import type { Invite } from './api';

const base: Invite = {
	id: 'i1',
	workspace_id: 'w1',
	email: 'person@example.com',
	role: 'member',
	token: 'abc',
	expires_at: '2026-01-01T00:00:00Z',
	accepted_at: null,
	created_at: '2025-12-01T00:00:00Z'
};

describe('inviteStatus', () => {
	it('prefers accepted over the clock', () => {
		expect(inviteStatus({ ...base, accepted_at: '2025-12-02T00:00:00Z' }, 0)).toBe('accepted');
	});

	it('marks past expiries as expired and future ones as pending', () => {
		expect(inviteStatus(base, Date.parse('2026-02-01T00:00:00Z'))).toBe('expired');
		expect(inviteStatus(base, Date.parse('2025-12-15T00:00:00Z'))).toBe('pending');
	});
});

describe('acceptErrorKind', () => {
	it('maps by status', () => {
		expect(acceptErrorKind(new ApiError(404, 'not_found', 'nope'))).toBe('unknown');
		expect(acceptErrorKind(new ApiError(403, 'forbidden', 'nope'))).toBe('wrong-email');
		expect(acceptErrorKind(new ApiError(401, 'unauthorized', 'nope'))).toBe('signed-out');
		expect(acceptErrorKind(new Error('boom'))).toBe('other');
	});

	it('splits the three 409 cases by message', () => {
		expect(acceptErrorKind(new ApiError(409, 'conflict', 'invite has expired'))).toBe('expired');
		expect(acceptErrorKind(new ApiError(409, 'conflict', 'invite has already been accepted'))).toBe(
			'accepted'
		);
		expect(
			acceptErrorKind(new ApiError(409, 'conflict', 'you are already a member of this workspace'))
		).toBe('member');
		expect(acceptErrorKind(new ApiError(409, 'conflict', 'something else'))).toBe('other');
	});
});
