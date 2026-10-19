import type { ResolvedPathname } from '$app/types';

/**
 * Only same-origin absolute paths are allowed after auth, so `?next=` can't be
 * used as an open redirect.
 */
export function safeNext(value: string | null, fallback: ResolvedPathname): ResolvedPathname {
	if (!value) return fallback;
	if (!value.startsWith('/') || value.startsWith('//')) return fallback;
	return value as ResolvedPathname;
}
