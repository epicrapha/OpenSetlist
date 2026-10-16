import type { SongListQuery } from './types';

/** Build the song list query string, dropping empty filters. Pure, so it is testable in node. */
export function songsQueryString(query: SongListQuery = {}): string {
	const params = new URLSearchParams();
	const add = (key: string, value: string | number | undefined) => {
		if (value === undefined || value === null) return;
		const text = `${value}`.trim();
		if (text !== '') params.set(key, text);
	};
	add('q', query.q);
	add('key', query.key);
	add('artist', query.artist);
	add('tag', query.tag);
	add('sort', query.sort);
	add('limit', query.limit);
	add('offset', query.offset);
	const search = params.toString();
	return search ? `?${search}` : '';
}
