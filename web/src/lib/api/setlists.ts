import { request } from './client';
import type {
	CreateSetlistInput,
	CreateSetlistItemInput,
	Setlist,
	SetlistDetail,
	SetlistItem,
	SetlistList,
	SetlistListQuery,
	UpdateSetlistInput,
	UpdateSetlistItemInput
} from './types';

/** Build the setlist list query string, dropping empty filters. */
export function setlistsQueryString(query: SetlistListQuery = {}): string {
	const params = new URLSearchParams();
	const add = (key: string, value: string | number | undefined) => {
		if (value === undefined || value === null) return;
		const text = `${value}`.trim();
		if (text !== '') params.set(key, text);
	};
	add('status', query.status);
	add('q', query.q);
	add('limit', query.limit);
	add('offset', query.offset);
	const search = params.toString();
	return search ? `?${search}` : '';
}

const base = '/setlists';
const itemPath = (setlistId: string, itemId: string) =>
	`${base}/${encodeURIComponent(setlistId)}/items/${encodeURIComponent(itemId)}`;

export const listSetlists = (
	workspaceId: string,
	query: SetlistListQuery = {},
	signal?: AbortSignal
): Promise<SetlistList> =>
	request<SetlistList>(`${base}${setlistsQueryString(query)}`, { workspaceId, signal });

export const getSetlist = (workspaceId: string, id: string): Promise<SetlistDetail> =>
	request<SetlistDetail>(`${base}/${encodeURIComponent(id)}`, { workspaceId });

export const createSetlist = (workspaceId: string, input: CreateSetlistInput): Promise<Setlist> =>
	request<Setlist>(base, { method: 'POST', body: input, workspaceId });

export const updateSetlist = (
	workspaceId: string,
	id: string,
	input: UpdateSetlistInput
): Promise<Setlist> =>
	request<Setlist>(`${base}/${encodeURIComponent(id)}`, {
		method: 'PATCH',
		body: input,
		workspaceId
	});

export const deleteSetlist = (workspaceId: string, id: string): Promise<void> =>
	request<void>(`${base}/${encodeURIComponent(id)}`, { method: 'DELETE', workspaceId });

export const duplicateSetlist = (workspaceId: string, id: string): Promise<Setlist> =>
	request<Setlist>(`${base}/${encodeURIComponent(id)}/duplicate`, {
		method: 'POST',
		workspaceId
	});

export const addItem = (
	workspaceId: string,
	setlistId: string,
	input: CreateSetlistItemInput
): Promise<SetlistItem> =>
	request<SetlistItem>(`${base}/${encodeURIComponent(setlistId)}/items`, {
		method: 'POST',
		body: input,
		workspaceId
	});

export const updateItem = (
	workspaceId: string,
	setlistId: string,
	itemId: string,
	input: UpdateSetlistItemInput
): Promise<SetlistItem> =>
	request<SetlistItem>(itemPath(setlistId, itemId), {
		method: 'PATCH',
		body: input,
		workspaceId
	});

export const deleteItem = (workspaceId: string, setlistId: string, itemId: string): Promise<void> =>
	request<void>(itemPath(setlistId, itemId), { method: 'DELETE', workspaceId });

export const reorderItems = (
	workspaceId: string,
	setlistId: string,
	itemIds: string[]
): Promise<SetlistItem[]> =>
	request<{ items: SetlistItem[] }>(`${base}/${encodeURIComponent(setlistId)}/items/order`, {
		method: 'PUT',
		body: { item_ids: itemIds },
		workspaceId
	}).then((data) => data.items);

export const shareSetlist = (workspaceId: string, id: string): Promise<{ share_token: string }> =>
	request<{ share_token: string }>(`${base}/${encodeURIComponent(id)}/share`, {
		method: 'POST',
		workspaceId
	});

export const unshareSetlist = (workspaceId: string, id: string): Promise<void> =>
	request<void>(`${base}/${encodeURIComponent(id)}/share`, { method: 'DELETE', workspaceId });

/** Public read: no workspace header, and a 401 must not bounce the visitor to /login. */
export const getSharedSetlist = (token: string): Promise<SetlistDetail> =>
	request<SetlistDetail>(`/shared/setlists/${encodeURIComponent(token)}`, { auth: false });
