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
	/** Multipart body; sent as-is so the browser sets the boundary header. */
	formData?: FormData;
	/** Extra headers, merged over the defaults. */
	headers?: Record<string, string>;
	/** Workspace scope: sets the `X-Workspace-Id` header the song routes require. */
	workspaceId?: string | null;
	/** false = a 401 is expected (login/register) and must not trigger the central handler. */
	auth?: boolean;
	signal?: AbortSignal;
}

async function send(path: string, options: RequestOptions = {}): Promise<Response> {
	const { method = 'GET', body, formData, headers, workspaceId, auth = true, signal } = options;

	const merged: Record<string, string> = { ...headers };
	if (workspaceId) merged['X-Workspace-Id'] = workspaceId;

	let payload: BodyInit | undefined;
	if (formData) {
		payload = formData;
	} else if (body !== undefined) {
		merged['content-type'] = 'application/json';
		payload = JSON.stringify(body);
	}

	let response: Response;
	try {
		response = await fetch(`${API_BASE}${path}`, {
			method,
			credentials: 'include',
			headers: Object.keys(merged).length > 0 ? merged : undefined,
			body: payload,
			signal
		});
	} catch {
		throw new ApiError(0, 'network_error', 'Could not reach the server.');
	}

	if (!response.ok) {
		if (response.status === 401 && auth) unauthorizedHandlers.forEach((handler) => handler());
		let data: unknown;
		try {
			const text = await response.text();
			data = text ? JSON.parse(text) : undefined;
		} catch {
			data = undefined;
		}
		const { code, message } = parseErrorBody(response.status, data);
		throw new ApiError(response.status, code, message);
	}

	return response;
}

/** Thin JSON wrapper: `credentials: 'include'`, error-shape parsing, central 401 handling. */
export async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
	const response = await send(path, options);
	if (response.status === 204) return undefined as T;

	const text = await response.text();
	let data: unknown;
	try {
		data = text ? JSON.parse(text) : undefined;
	} catch {
		data = undefined;
	}

	return data as T;
}

/** Raw response for non-JSON payloads (text export, PDF download). Same error handling. */
export async function requestResponse(
	path: string,
	options: RequestOptions = {}
): Promise<Response> {
	return send(path, options);
}
