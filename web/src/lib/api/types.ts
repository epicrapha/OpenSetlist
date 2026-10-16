/** Domain types for the M0 API contract (PLAN.md §3). */

export type Role = 'owner' | 'admin' | 'leader' | 'member' | 'viewer';

export interface User {
	id: string;
	email: string;
	name: string;
	created_at?: string;
}

export interface Membership {
	id: string;
	user_id: string;
	workspace_id: string;
	role: Role;
}

export interface Workspace {
	id: string;
	name: string;
	plan?: string;
	settings?: unknown;
	created_at?: string;
}

/** `GET /api/v1/auth/me` — current user + memberships. */
export interface Me {
	user: User;
	memberships: Membership[];
}

/** `POST /api/v1/workspaces` — workspace + owner membership. */
export interface WorkspaceCreated {
	workspace: Workspace;
	membership: Membership;
}

/** Song JSON (PLAN.md §3.1). */
export interface Song {
	id: string;
	workspace_id: string;
	title: string;
	artist: string | null;
	default_key: string | null;
	tempo: number | null;
	time_signature: string;
	tags: string[];
	ccli_number: string | null;
	copyright: string | null;
	chordpro: string;
	created_at: string;
	updated_at: string;
}

/** `GET /api/v1/songs` — page of songs plus the total matching the filters. */
export interface SongList {
	songs: Song[];
	total: number;
}

export type SongSort = 'title' | 'updated_at';

export interface SongListQuery {
	q?: string;
	key?: string;
	artist?: string;
	tag?: string;
	sort?: SongSort;
	limit?: number;
	offset?: number;
}

export interface CreateSongInput {
	title: string;
	artist?: string | null;
	default_key?: string | null;
	tempo?: number | null;
	time_signature?: string | null;
	tags?: string[] | null;
	ccli_number?: string | null;
	copyright?: string | null;
	chordpro: string;
}

/** Partial update; omit a key to leave it unchanged, send `null` to clear it. */
export type UpdateSongInput = Partial<CreateSongInput>;

export interface Arrangement {
	id: string;
	song_id: string;
	name: string;
	key: string | null;
	chordpro: string;
}

export interface CreateArrangementInput {
	name: string;
	key?: string | null;
	chordpro?: string;
}

export type UpdateArrangementInput = Partial<CreateArrangementInput>;

export type AssetKind = 'link' | 'pdf';

export interface Asset {
	id: string;
	song_id: string;
	kind: AssetKind;
	url: string | null;
	label: string | null;
	filename: string | null;
	size_bytes: number | null;
	content_type: string | null;
	created_at: string;
}

export type ExportFormat = 'chordpro' | 'text';
