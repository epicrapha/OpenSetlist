import { PUBLIC_API_BASE_URL } from '$app/env/public';
import { ApiError, parseErrorBody } from './errors';

/** Absolute base if configured, otherwise same-origin. */
const BASE = (PUBLIC_API_BASE_URL ?? '').replace(/\/+$/, '');
export const API_BASE = `${BASE}/api/v1`;

type UnauthorizedHandler = () => void;
const unauthorizedHandlers = new Set<UnauthorizedHandler>();

/** Register a central 401 handler (e.g. redirect to /login). Returns an unsubscribe. */
export function onUnauthorized(handler: UnauthorizedHandler): () => void {
	unauthorizedHandlers.add(handler);
	return () => unauthorizedHandlers.delete(handler);
}

export interface RequestOptions {
	method?: 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE';
	body?: unknown;
	/** false = a 401 is expected (login/register) and must not trigger the central handler. */
	auth?: boolean;
	signal?: AbortSignal;
}

/** Thin JSON wrapper: `credentials: 'include'`, error-shape parsing, central 401 handling. */
export async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
	const { method = 'GET', body, auth = true, signal } = options;

	let response: Response;
	try {
		response = await fetch(`${API_BASE}${path}`, {
			method,
			credentials: 'include',
			headers: body === undefined ? undefined : { 'content-type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body),
			signal
		});
	} catch {
		throw new ApiError(0, 'network_error', 'Could not reach the server.');
	}

	if (response.status === 204) return undefined as T;

	const text = await response.text();
	let data: unknown;
	try {
		data = text ? JSON.parse(text) : undefined;
	} catch {
		data = undefined;
	}

	if (!response.ok) {
		if (response.status === 401 && auth) unauthorizedHandlers.forEach((handler) => handler());
		const { code, message } = parseErrorBody(response.status, data);
		throw new ApiError(response.status, code, message);
	}

	return data as T;
}
