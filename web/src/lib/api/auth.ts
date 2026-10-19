import { request } from './client';
import type { Me, User } from './types';

export interface RegisterInput {
	email: string;
	password: string;
	name: string;
}

export interface LoginInput {
	email: string;
	password: string;
}

// NOTE: register/login body shape is not pinned by the contract; callers here only
// care about success + the session cookie. Reconcile the type when the server lands.
export const register = (input: RegisterInput): Promise<User> =>
	request<User>('/auth/register', { method: 'POST', body: input, auth: false });

export const login = (input: LoginInput): Promise<User> =>
	request<User>('/auth/login', { method: 'POST', body: input, auth: false });

export const logout = (): Promise<void> => request<void>('/auth/logout', { method: 'POST' });

/** Current user + memberships; throws ApiError(401) when the session is gone. */
export const me = (options: { auth?: boolean } = {}): Promise<Me> =>
	request<Me>('/auth/me', { auth: options.auth ?? true });
