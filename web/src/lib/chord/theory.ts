import type { Accidental } from './types';

const SHARP_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
const FLAT_NAMES = ['C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab', 'A', 'Bb', 'B'];
const LETTER_PC: Record<string, number> = { C: 0, D: 2, E: 4, F: 5, G: 7, A: 9, B: 11 };

export function mod12(n: number): number {
	return ((n % 12) + 12) % 12;
}

/** Pitch class (0–11) of a note name like `C`, `F#`, `Bb`; null if unparseable. */
export function noteToPc(note: string): number | null {
	const match = /^([A-Ga-g])([#b]?)$/.exec(note.trim());
	if (!match) return null;
	let pc = LETTER_PC[match[1].toUpperCase()];
	if (match[2] === '#') pc += 1;
	else if (match[2] === 'b') pc -= 1;
	return mod12(pc);
}

export function spell(pc: number, prefer: Accidental): string {
	return (prefer === 'flat' ? FLAT_NAMES : SHARP_NAMES)[mod12(pc)];
}

export function transposeNote(note: string, semitones: number, prefer: Accidental): string {
	const pc = noteToPc(note);
	return pc === null ? note : spell(pc + semitones, prefer);
}

export interface ParsedKey {
	pc: number;
	minor: boolean;
	root: string;
}

export function parseKey(key: string): ParsedKey | null {
	const match = /^\s*([A-Ga-g][#b]?)\s*([A-Za-z]*)\s*$/.exec(key);
	if (!match) return null;
	const root = match[1][0].toUpperCase() + match[1].slice(1);
	const pc = noteToPc(root);
	if (pc === null) return null;
	const suffix = match[2].toLowerCase();
	if (suffix === '' || suffix === 'maj' || suffix === 'major') return { pc, minor: false, root };
	if (suffix === 'm' || suffix === 'min' || suffix === 'minor') return { pc, minor: true, root };
	return null;
}

/** Canonical short form, e.g. `A minor` -> `Am`, preserving the author's accidental. */
export function normalizeKey(key: string): string | null {
	const parsed = parseKey(key);
	return parsed ? parsed.root + (parsed.minor ? 'm' : '') : null;
}

// Key-signature spelling preference by pitch class (circle of fifths).
const MAJOR_PREF: Accidental[] = [
	'sharp',
	'flat',
	'sharp',
	'flat',
	'sharp',
	'flat',
	'sharp',
	'sharp',
	'flat',
	'sharp',
	'flat',
	'sharp'
];
const MINOR_PREF: Accidental[] = [
	'flat',
	'sharp',
	'flat',
	'flat',
	'sharp',
	'flat',
	'sharp',
	'flat',
	'sharp',
	'sharp',
	'flat',
	'sharp'
];

export function keyAccidental(pc: number, minor: boolean): Accidental {
	return (minor ? MINOR_PREF : MAJOR_PREF)[mod12(pc)];
}

const MAJOR_STEPS = [0, 2, 4, 5, 7, 9, 11];
const MINOR_STEPS = [0, 2, 3, 5, 7, 8, 10];

/** pc -> note name for the seven diatonic pitches of a key (correct enharmonics). */
export function scaleSpelling(rootPc: number, minor: boolean): Map<number, string> {
	const prefer = keyAccidental(rootPc, minor);
	const map = new Map<number, string>();
	for (const step of minor ? MINOR_STEPS : MAJOR_STEPS) {
		const pc = mod12(rootPc + step);
		map.set(pc, spell(pc, prefer));
	}
	return map;
}
