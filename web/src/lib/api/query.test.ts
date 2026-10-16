import { describe, expect, it } from 'vitest';
import { songsQueryString } from './query';

describe('songsQueryString', () => {
	it('returns an empty string when nothing is set', () => {
		expect(songsQueryString()).toBe('');
		expect(songsQueryString({ q: '  ', key: '', limit: undefined })).toBe('');
	});

	it('includes filters, sort, and paging in order', () => {
		const result = songsQueryString({
			q: 'grace',
			key: 'G',
			sort: 'updated_at',
			limit: 20,
			offset: 40
		});
		expect(result).toBe('?q=grace&key=G&sort=updated_at&limit=20&offset=40');
	});

	it('encodes values with spaces and symbols', () => {
		expect(songsQueryString({ q: 'amazing grace' })).toBe('?q=amazing+grace');
		expect(songsQueryString({ artist: 'Matt & Beth' })).toBe('?artist=Matt+%26+Beth');
	});
});
