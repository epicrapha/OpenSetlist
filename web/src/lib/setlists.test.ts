import { describe, expect, it } from 'vitest';
import type { SetlistItem } from './api/types';
import {
	formatDuration,
	itemDuration,
	itemHints,
	runHints,
	signedKeyDistance,
	totalDuration
} from './setlists';

function item(overrides: Partial<SetlistItem> = {}): SetlistItem {
	return {
		id: overrides.id ?? 'item-1',
		setlist_id: 'setlist-1',
		position: 0,
		kind: 'song',
		song_id: 'song-1',
		title: null,
		notes: null,
		key_override: null,
		capo_override: null,
		duration_seconds: null,
		song: {
			id: 'song-1',
			title: 'Song',
			artist: null,
			default_key: 'G',
			tempo: 120,
			time_signature: '4/4',
			chordpro: '{key: G}\n[G]Grace'
		},
		...overrides
	};
}

describe('setlist duration', () => {
	it('defaults songs to 4 minutes and notes to 1', () => {
		expect(itemDuration(item())).toBe(240);
		expect(itemDuration(item({ kind: 'note', song: null, title: 'Prayer' }))).toBe(60);
	});

	it('honours an override, including an explicit zero', () => {
		expect(itemDuration(item({ duration_seconds: 305 }))).toBe(305);
		expect(itemDuration(item({ duration_seconds: 0 }))).toBe(0);
	});

	it('sums mixed items', () => {
		expect(
			totalDuration([
				item(),
				item({ kind: 'note', song: null, title: 'Scripture' }),
				item({ duration_seconds: 120 })
			])
		).toBe(420);
	});

	it('formats minutes and hours', () => {
		expect(formatDuration(0)).toBe('0 min');
		expect(formatDuration(90)).toBe('2 min');
		expect(formatDuration(3600)).toBe('1 hr');
		expect(formatDuration(3900)).toBe('1 hr 5 min');
	});
});

describe('setlist hints', () => {
	it('measures keys by the shortest signed distance', () => {
		expect(signedKeyDistance('G', 'A')).toBe(2);
		expect(signedKeyDistance('A', 'G')).toBe(-2);
		expect(signedKeyDistance('B', 'C')).toBe(1);
		expect(signedKeyDistance('C', 'B')).toBe(-1);
	});

	it('calls out a key change and a tempo jump', () => {
		const previous = item({ id: 'a' });
		const current = item({
			id: 'b',
			song: { ...item().song!, default_key: 'A', tempo: 160 }
		});
		expect(itemHints(current, previous)).toEqual([
			'Key change: G → A (+2 semitones).',
			'Tempo up 40 bpm (120 → 160).'
		]);
	});

	it('notes a repeated key and ignores small tempo drift', () => {
		const previous = item({ id: 'a' });
		const current = item({ id: 'b', song: { ...item().song!, tempo: 128 } });
		expect(itemHints(current, previous)).toEqual(['Same key as the previous song (G).']);
	});

	it('uses the item override and skips notes in the run', () => {
		const first = item({ id: 'a' });
		const prayer = item({ id: 'p', kind: 'note', song: null, title: 'Prayer' });
		const second = item({
			id: 'b',
			key_override: 'Bb',
			song: { ...item().song!, tempo: 90 }
		});
		expect(itemHints(second, prayer)).toEqual([]);
		const hints = runHints([first, prayer, second]);
		expect(hints.get('b')).toEqual([
			'Key change: G → Bb (+3 semitones).',
			'Tempo down 30 bpm (120 → 90).'
		]);
	});
});
