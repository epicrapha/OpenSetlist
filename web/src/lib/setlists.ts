import type { Setlist, SetlistItem, SetlistStatus } from './api/types';
import { capo, detectKey, parseChordPro, transpose } from './chord';
import type { Chart } from './chord';
import { mod12, parseKey } from './chord/theory';

/** SET-6 defaults: a song is four minutes, a spoken/other item is one. */
export const SONG_SECONDS = 240;
export const NOTE_SECONDS = 60;

export const setlistStatusLabels: Record<SetlistStatus, string> = {
	draft: 'Draft',
	ready: 'Ready',
	archived: 'Archived'
};

export const notePresets = ['Prayer', 'Scripture', 'Offering', 'Transition'] as const;

export function defaultDuration(kind: SetlistItem['kind']): number {
	return kind === 'song' ? SONG_SECONDS : NOTE_SECONDS;
}

/** Item length with the per-item override, falling back to the kind default. */
export function itemDuration(item: Pick<SetlistItem, 'kind' | 'duration_seconds'>): number {
	const value = item.duration_seconds;
	return typeof value === 'number' && Number.isFinite(value) && value >= 0
		? value
		: defaultDuration(item.kind);
}

export function totalDuration(items: Pick<SetlistItem, 'kind' | 'duration_seconds'>[]): number {
	return items.reduce((sum, item) => sum + itemDuration(item), 0);
}

/** Human duration for the run sheet: `42 min`, `1 hr 5 min`. */
export function formatDuration(seconds: number): string {
	const safe = Number.isFinite(seconds) ? Math.max(0, Math.round(seconds)) : 0;
	const hours = Math.floor(safe / 3600);
	const minutes = Math.round((safe % 3600) / 60);
	if (hours === 0) return `${minutes} min`;
	return minutes === 0 ? `${hours} hr` : `${hours} hr ${minutes} min`;
}

/** `YYYY-MM-DD` from the API -> a readable date without a timezone shift. */
export function formatServiceDate(date: string | null | undefined): string {
	if (!date) return 'No date yet';
	const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date);
	if (!match) return date;
	const value = new Date(Number(match[1]), Number(match[2]) - 1, Number(match[3]));
	return value.toLocaleDateString(undefined, {
		weekday: 'short',
		day: 'numeric',
		month: 'short',
		year: 'numeric'
	});
}

export function itemTitle(item: SetlistItem): string {
	return item.title?.trim() || item.song?.title || 'Untitled item';
}

/** The key a song is actually played/sung in, honouring the item override. */
export function itemKey(item: SetlistItem): string | null {
	return item.key_override || item.song?.default_key || null;
}

export function itemTempo(item: SetlistItem): number | null {
	return item.song?.tempo ?? null;
}

/** Signed nearest distance between two keys, e.g. G -> A = +2, A -> G = -2. */
export function signedKeyDistance(from: string, to: string): number | null {
	const a = parseKey(from);
	const b = parseKey(to);
	if (!a || !b) return null;
	let delta = mod12(b.pc - a.pc);
	if (delta > 6) delta -= 12;
	return delta;
}

function keyHint(current: SetlistItem, previous: SetlistItem): string | null {
	const from = itemKey(previous);
	const to = itemKey(current);
	if (!from || !to) return null;

	const a = parseKey(from);
	const b = parseKey(to);
	if (!a || !b) return from === to ? `Same key as the previous song (${to}).` : null;

	if (a.pc === b.pc) {
		if (a.minor === b.minor) return `Same key as the previous song (${to}).`;
		return `Key quality changes: ${from} → ${to} (same root).`;
	}

	const distance = signedKeyDistance(from, to);
	if (distance === null) return null;
	const amount = Math.abs(distance);
	return `Key change: ${from} → ${to} (${distance > 0 ? '+' : '−'}${amount} semitone${amount === 1 ? '' : 's'}).`;
}

function tempoHint(current: SetlistItem, previous: SetlistItem): string | null {
	const from = itemTempo(previous);
	const to = itemTempo(current);
	if (from === null || to === null || from === to) return null;
	const diff = to - from;
	// Only call out a meaningful jump; a 2 bpm drift is noise.
	if (Math.abs(diff) < 20) return null;
	return `Tempo ${diff > 0 ? 'up' : 'down'} ${Math.abs(diff)} bpm (${from} → ${to}).`;
}

/**
 * Gentle transition hints, computed client-side (SET-10). Notes are skipped so
 * a prayer between two songs does not hide the musical transition.
 */
export function itemHints(current: SetlistItem, previous: SetlistItem | null): string[] {
	if (!previous || current.kind !== 'song' || previous.kind !== 'song') return [];
	return [keyHint(current, previous), tempoHint(current, previous)].filter(
		(hint): hint is string => hint !== null
	);
}

/** Item id -> hints relative to the previous *song* item. */
export function runHints(items: SetlistItem[]): Map<string, string[]> {
	const hints = new Map<string, string[]>();
	let previousSong: SetlistItem | null = null;
	for (const item of items) {
		if (item.kind !== 'song') continue;
		const messages = itemHints(item, previousSong);
		if (messages.length > 0) hints.set(item.id, messages);
		previousSong = item;
	}
	return hints;
}

export interface ItemChart {
	chart: Chart;
	/** Key the band sounds in. */
	soundingKey: string;
	/** Shapes/key to play once the capo is taken into account. */
	shapeKey: string;
	capoFret: number;
}

/**
 * Chart for a setlist item with its overrides applied. This never touches the
 * stored song: key and capo are view-time transforms on a fresh parse.
 */
export function itemChart(item: SetlistItem): ItemChart | null {
	if (!item.song) return null;
	const base = parseChordPro(item.song.chordpro);
	const sourceKey = item.song.default_key ?? detectKey(base);
	const targetKey = item.key_override || sourceKey;
	let chart = base;

	if (targetKey && sourceKey) {
		const from = parseKey(sourceKey);
		const to = parseKey(targetKey);
		if (from && to) chart = transpose(base, mod12(to.pc - from.pc));
	}

	const fret = Math.max(0, item.capo_override ?? 0);
	if (fret > 0) {
		const shaped = capo(chart, fret);
		return {
			chart: shaped.shapes,
			soundingKey: shaped.soundingKey,
			shapeKey: shaped.shapeKey,
			capoFret: fret
		};
	}

	const soundingKey = detectKey(chart) ?? targetKey ?? '';
	return {
		chart,
		soundingKey,
		shapeKey: soundingKey,
		capoFret: 0
	};
}

export function itemCountLabel(setlist: Pick<Setlist, 'item_count'>): string {
	const count = setlist.item_count;
	return count === 1 ? '1 item' : `${count} items`;
}
