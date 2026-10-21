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

/** Invite roles never include `owner` (PLAN.md §3.2). */
export type InviteRole = 'admin' | 'leader' | 'member' | 'viewer';

export interface Invite {
	id: string;
	workspace_id: string;
	email: string;
	role: InviteRole;
	token: string;
	expires_at: string;
	accepted_at: string | null;
	created_at: string;
}

/** `POST /api/v1/invites/:token/accept`. */
export interface AcceptInviteResult {
	workspace_id: string;
	role: Role;
}

/** Setlist status (PLAN.md §3.3). Archived setlists are read-only. */
export type SetlistStatus = 'draft' | 'ready' | 'archived';

export type SetlistItemKind = 'song' | 'note';

/** Embedded song fields on a setlist item — the chart source, never the whole song. */
export interface SetlistSong {
	id: string;
	title: string;
	artist: string | null;
	default_key: string | null;
	tempo: number | null;
	time_signature: string;
	chordpro: string;
}

/** Setlist header JSON (PLAN.md §3.3). */
export interface Setlist {
	id: string;
	workspace_id: string;
	title: string;
	service_date: string | null;
	service_type: string | null;
	status: SetlistStatus;
	is_template: boolean;
	share_token: string | null;
	item_count: number;
	created_at: string;
	updated_at: string;
}

/** Setlist item JSON. Overrides live on the item and never touch the song. */
export interface SetlistItem {
	id: string;
	setlist_id: string;
	position: number;
	kind: SetlistItemKind;
	song_id: string | null;
	title: string | null;
	notes: string | null;
	key_override: string | null;
	/** Null means "use the library value / no capo". */
	capo_override: number | null;
	/** Null means "use the default for this kind" (songs 240 s, notes 60 s). */
	duration_seconds: number | null;
	song: SetlistSong | null;
}

export interface SetlistList {
	setlists: Setlist[];
	total: number;
}

export interface SetlistListQuery {
	status?: SetlistStatus | '';
	q?: string;
	limit?: number;
	offset?: number;
}

/** `GET /api/v1/setlists/:id` and the public share shape. */
export interface SetlistDetail {
	setlist: Setlist;
	items: SetlistItem[];
}

export interface CreateSetlistInput {
	title: string;
	service_date?: string | null;
	service_type?: string | null;
}

/** Omit a key to leave it unchanged; send `null` to clear it. */
export interface UpdateSetlistInput {
	title?: string;
	service_date?: string | null;
	service_type?: string | null;
	status?: SetlistStatus;
	is_template?: boolean;
}

export interface CreateSetlistItemInput {
	kind: SetlistItemKind;
	song_id?: string;
	title?: string | null;
	notes?: string | null;
	key_override?: string | null;
	capo_override?: number | null;
	duration_seconds?: number | null;
	position?: number;
}

export type UpdateSetlistItemInput = Partial<Omit<CreateSetlistItemInput, 'kind'>>;

/** `POST /api/v1/setlists/:id/share`. */
export interface ShareResult {
	share_token: string;
}
