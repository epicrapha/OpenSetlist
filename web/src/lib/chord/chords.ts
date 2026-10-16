export interface ParsedChord {
	root: string;
	quality: string;
	bass: string | null;
}

const NO_CHORD = /^N\.?C?\.?$|^N\/C$/i;
// Characters legal in a chord suffix: m/Maj/min/dim/aug/sus/add/no/omit/alt, extensions,
// alterations and 6/9-style slashes. Rejects arbitrary words ("mazing", "ather").
const QUALITY_CHARS = new Set('0123456789mMajindugseotblM#b\u00b0\u00f8+()/-');

export function isNoChord(token: string): boolean {
	return NO_CHORD.test(token);
}

function validQuality(quality: string): boolean {
	for (const ch of quality) if (!QUALITY_CHARS.has(ch)) return false;
	return true;
}

/** Parse a chord token into root/quality/bass; null when the token is not a chord. */
export function parseChordToken(token: string): ParsedChord | null {
	if (isNoChord(token)) return null;
	const rootMatch = /^([A-G][#b]?)/.exec(token);
	if (!rootMatch) return null;
	const root = rootMatch[1];
	let rest = token.slice(root.length);
	let bass: string | null = null;

	// A trailing `/<note>` is a slash bass; `/9` (6/9 chords) stays in the quality.
	const bassMatch = /^(.*)\/([A-G][#b]?)$/.exec(rest);
	if (bassMatch) {
		rest = bassMatch[1];
		bass = bassMatch[2];
	}

	if (!validQuality(rest)) return null;
	return { root, quality: rest, bass };
}

export function isChordToken(token: string): boolean {
	return isNoChord(token) || parseChordToken(token) !== null;
}

export type NoteSpeller = (note: string) => string;

/** Transpose a chord token, respelling the root and bass; `N.C.` is left untouched. */
export function transposeChord(token: string, spellNote: NoteSpeller): string {
	if (isNoChord(token)) return token;
	const parsed = parseChordToken(token);
	if (!parsed) return token;
	const root = spellNote(parsed.root);
	const bass = parsed.bass ? '/' + spellNote(parsed.bass) : '';
	return root + parsed.quality + bass;
}
