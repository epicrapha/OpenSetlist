/** Error wire shape from PLAN.md §3: `{ "error": { "code", "message" } }`. */
export interface ApiErrorBody {
	error: { code: string; message: string };
}

/** Thrown for any non-2xx response or a transport failure. */
export class ApiError extends Error {
	readonly code: string;
	readonly status: number;

	constructor(status: number, code: string, message: string) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
		this.code = code;
	}
}

/** Extract `{ code, message }` from an unknown error body, with a safe fallback. */
export function parseErrorBody(status: number, data: unknown): { code: string; message: string } {
	const error = (data as ApiErrorBody | undefined)?.error;
	if (error && typeof error.code === 'string' && typeof error.message === 'string') {
		return { code: error.code, message: error.message };
	}
	return { code: `http_${status}`, message: `Request failed with status ${status}.` };
}
