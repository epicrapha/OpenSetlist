import type { Role } from './api';

const WRITE_ROLES = new Set<Role>(['owner', 'admin', 'leader']);

/** Writes require `leader` or above (PLAN.md §3.1). */
export function canWrite(role: Role | null | undefined): boolean {
	return role !== null && role !== undefined && WRITE_ROLES.has(role);
}
