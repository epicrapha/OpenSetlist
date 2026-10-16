import { request, requestResponse } from './client';
import { songsQueryString } from './query';
import type {
	Arrangement,
	Asset,
	CreateArrangementInput,
	CreateSongInput,
	ExportFormat,
	Song,
	SongList,
	SongListQuery,
	UpdateArrangementInput,
	UpdateSongInput
} from './types';

export const listSongs = (
	workspaceId: string,
	query: SongListQuery = {},
	signal?: AbortSignal
): Promise<SongList> =>
	request<SongList>(`/songs${songsQueryString(query)}`, { workspaceId, signal });

export const getSong = (workspaceId: string, id: string): Promise<Song> =>
	request<Song>(`/songs/${encodeURIComponent(id)}`, { workspaceId });

export const createSong = (workspaceId: string, input: CreateSongInput): Promise<Song> =>
	request<Song>('/songs', { method: 'POST', body: input, workspaceId });

export const updateSong = (
	workspaceId: string,
	id: string,
	input: UpdateSongInput
): Promise<Song> =>
	request<Song>(`/songs/${encodeURIComponent(id)}`, {
		method: 'PATCH',
		body: input,
		workspaceId
	});

export const deleteSong = (workspaceId: string, id: string): Promise<void> =>
	request<void>(`/songs/${encodeURIComponent(id)}`, { method: 'DELETE', workspaceId });

export const duplicateSong = (workspaceId: string, id: string): Promise<Song> =>
	request<Song>(`/songs/${encodeURIComponent(id)}/duplicate`, { method: 'POST', workspaceId });

/** Returns the raw export response; fold it into a download with `saveResponse`. */
export const exportSong = (
	workspaceId: string,
	id: string,
	format: ExportFormat
): Promise<Response> =>
	requestResponse(`/songs/${encodeURIComponent(id)}/export?format=${format}`, { workspaceId });

export const listArrangements = (workspaceId: string, songId: string): Promise<Arrangement[]> =>
	request<Arrangement[]>(`/songs/${encodeURIComponent(songId)}/arrangements`, { workspaceId });

export const createArrangement = (
	workspaceId: string,
	songId: string,
	input: CreateArrangementInput
): Promise<Arrangement> =>
	request<Arrangement>(`/songs/${encodeURIComponent(songId)}/arrangements`, {
		method: 'POST',
		body: input,
		workspaceId
	});

export const updateArrangement = (
	workspaceId: string,
	id: string,
	input: UpdateArrangementInput
): Promise<Arrangement> =>
	request<Arrangement>(`/arrangements/${encodeURIComponent(id)}`, {
		method: 'PATCH',
		body: input,
		workspaceId
	});

export const deleteArrangement = (workspaceId: string, id: string): Promise<void> =>
	request<void>(`/arrangements/${encodeURIComponent(id)}`, { method: 'DELETE', workspaceId });

export const listAssets = async (workspaceId: string, songId: string): Promise<Asset[]> =>
	(
		await request<{ assets: Asset[] }>(`/songs/${encodeURIComponent(songId)}/assets`, {
			workspaceId
		})
	).assets;

export const createLink = (
	workspaceId: string,
	songId: string,
	url: string,
	label?: string
): Promise<Asset> =>
	request<Asset>(`/songs/${encodeURIComponent(songId)}/assets`, {
		method: 'POST',
		body: { kind: 'link', url, label },
		workspaceId
	});

export const uploadPdf = (workspaceId: string, songId: string, file: File): Promise<Asset> => {
	const formData = new FormData();
	formData.append('file', file);
	return request<Asset>(`/songs/${encodeURIComponent(songId)}/assets/pdf`, {
		method: 'POST',
		formData,
		workspaceId
	});
};

export const deleteAsset = (workspaceId: string, id: string): Promise<void> =>
	request<void>(`/assets/${encodeURIComponent(id)}`, { method: 'DELETE', workspaceId });

/** Raw PDF download; fold it into a download with `saveResponse`. */
export const downloadAsset = (workspaceId: string, id: string): Promise<Response> =>
	requestResponse(`/assets/${encodeURIComponent(id)}/file`, { workspaceId });
