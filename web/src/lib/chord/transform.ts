import { parseChordToken, transposeChord, type NoteSpeller } from './chords';
import {
	keyAccidental,
	mod12,
	normalizeKey,
	noteToPc,
	parseKey,
	scaleSpelling,
	spell
} from './theory';
import type { Accidental, Chart } from './types';

function cloneChart(chart: Chart): Chart {
	return JSON.parse(JSON.stringify(chart)) as Chart;
}

function chordTokens(chart: Chart): string[] {
	const tokens: string[] = [];
	for (const section of chart.sections) {
		for (const line of section.lines) {
			if (line.kind !== 'chord') continue;
			for (const segment of line.segments) if (segment.chord) tokens.push(segment.chord);
		}
	}
	return tokens;
}

function isMinorQuality(quality: string): boolean {
	return /^m(?!aj)/.test(quality) || /^min/.test(quality);
}

/** Spellings already present: sharps vs flats, used only when there is no key to guide us. */
function inferAccidental(chart: Chart): Accidental {
	let sharp = 0;
	let flat = 0;
	for (const token of chordTokens(chart)) {
		const parsed = parseChordToken(token);
		if (!parsed) continue;
		for (const note of [parsed.root, parsed.bass]) {
			if (!note) continue;
			if (note.includes('#')) sharp += 1;
			if (note.includes('b')) flat += 1;
		}
	}
	return flat > sharp ? 'flat' : 'sharp';
}

function chartKey(chart: Chart): { pc: number; minor: boolean } | null {
	const fromMeta = chart.meta.key ? parseKey(chart.meta.key) : null;
	if (fromMeta) return { pc: fromMeta.pc, minor: fromMeta.minor };
	const detected = detectKey(chart);
	if (!detected) return null;
	const parsed = parseKey(detected);
	return parsed ? { pc: parsed.pc, minor: parsed.minor } : null;
}

export function detectKey(chart: Chart): string | null {
	if (chart.meta.key) return normalizeKey(chart.meta.key);

	const tally = new Map<number, { count: number; minor: number; order: number }>();
	let order = 0;
	for (const token of chordTokens(chart)) {
		const parsed = parseChordToken(token);
		if (!parsed) continue;
		const pc = noteToPc(parsed.root);
		if (pc === null) continue;
		const entry = tally.get(pc) ?? { count: 0, minor: 0, order: order++ };
		entry.count += 1;
		if (isMinorQuality(parsed.quality)) entry.minor += 1;
		tally.set(pc, entry);
	}
	if (tally.size === 0) return null;

	let bestPc = -1;
	let best: { count: number; minor: number; order: number } | null = null;
	for (const [pc, entry] of tally) {
		if (
			!best ||
			entry.count > best.count ||
			(entry.count === best.count && entry.order < best.order)
		) {
			best = entry;
			bestPc = pc;
		}
	}
	if (!best) return null;

	const minor = best.minor * 2 > best.count;
	return spell(bestPc, keyAccidental(bestPc, minor)) + (minor ? 'm' : '');
}

export function transposeKey(key: string, semitones: number, prefer?: Accidental): string {
	const parsed = parseKey(key);
	if (!parsed) return key;
	const shift = mod12(semitones);
	if (shift === 0 && !prefer) return key;
	const pc = mod12(parsed.pc + shift);
	const accidental = prefer ?? keyAccidental(pc, parsed.minor);
	return spell(pc, accidental) + (parsed.minor ? 'm' : '');
}

export function transpose(chart: Chart, semitones: number, prefer?: Accidental): Chart {
	const shift = mod12(semitones);
	if (shift === 0 && !prefer) return cloneChart(chart);

	const key = chartKey(chart);
	const targetPc = key ? mod12(key.pc + shift) : 0;
	const accidental: Accidental =
		prefer ?? (key ? keyAccidental(targetPc, key.minor) : inferAccidental(chart));
	// Diatonic notes use the target key's scale spelling; explicit `prefer` forces a pure
	// chromatic respelling, and keyless charts fall back to the inferred accidental.
	const scale = !prefer && key ? scaleSpelling(targetPc, key.minor) : null;
	const spellNote: NoteSpeller = (note) => {
		const pc = noteToPc(note);
		if (pc === null) return note;
		const shifted = mod12(pc + shift);
		return scale?.get(shifted) ?? spell(shifted, accidental);
	};

	const next = cloneChart(chart);
	for (const section of next.sections) {
		for (const line of section.lines) {
			if (line.kind !== 'chord') continue;
			for (const segment of line.segments) {
				if (segment.chord) segment.chord = transposeChord(segment.chord, spellNote);
			}
		}
	}
	if (next.meta.key) next.meta.key = transposeKey(next.meta.key, shift, accidental);

	return next;
}

export interface CapoResult {
	shapes: Chart;
	shapeKey: string;
	soundingKey: string;
}

/** Capo at `fret`: chord shapes are the sounding chords transposed down by `fret`. */
export function capo(chart: Chart, fret: number, prefer?: Accidental): CapoResult {
	const soundingKey = detectKey(chart) ?? '';
	const shapes = transpose(chart, -fret, prefer);
	const shapeKey = soundingKey ? transposeKey(soundingKey, -fret, prefer) : '';
	return { shapes, shapeKey, soundingKey };
}
