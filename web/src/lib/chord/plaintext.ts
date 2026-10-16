import { isChordToken } from './chords';

const TOKEN_RE = /\S+/g;

const LABEL_RE =
	/^(verse|chorus|bridge|pre-?chorus|intro|outro|tag|instrumental|solo|interlude|vamp|refrain|ending|turnaround|coda)(\s*\d+)?\s*:?$/i;
const BRACKET_LABEL_RE = /^\[([^\]]+)\]$/;

interface PositionedChord {
	chord: string;
	index: number;
}

function sectionLabel(line: string): string | null {
	const trimmed = line.trim();
	const bracket = BRACKET_LABEL_RE.exec(trimmed);
	if (bracket && !isChordToken(bracket[1].trim())) return bracket[1].trim();
	if (LABEL_RE.test(trimmed)) return trimmed.replace(/\s*:\s*$/, '');
	return null;
}

function positionedChords(line: string): PositionedChord[] {
	const out: PositionedChord[] = [];
	for (const match of line.matchAll(TOKEN_RE)) {
		const token = match[0];
		if (token === '|' || token === '||') continue;
		if (isChordToken(token)) out.push({ chord: token, index: match.index });
	}
	return out;
}

function isChordLine(line: string): boolean {
	const tokens = line
		.split(/\s+/)
		.filter(Boolean)
		.filter((token) => token !== '|' && token !== '||');
	return tokens.length > 0 && tokens.every(isChordToken);
}

/** Insert chords as ChordPro `[chord]` markers at their original columns. */
function mergeChordLine(chords: PositionedChord[], lyric: string): string {
	let result = '';
	let cursor = 0;

	for (const { chord, index } of chords) {
		const column = Math.min(index, lyric.length);
		if (column > cursor) {
			result += lyric.slice(cursor, column);
			cursor = column;
		} else if (
			column === cursor &&
			cursor === lyric.length &&
			result !== '' &&
			!result.endsWith(' ')
		) {
			result += ' ';
		}
		result += `[${chord}]`;
	}

	result += lyric.slice(cursor);
	return result;
}

/** Convert a plain "chords above lyrics" chart into ChordPro. */
export function fromPlainText(source: string): string {
	const lines = source.replace(/\r\n?/g, '\n').split('\n');
	const out: string[] = [];

	for (let i = 0; i < lines.length; i += 1) {
		const line = lines[i];

		if (line.trim() === '') {
			out.push('');
			continue;
		}

		const label = sectionLabel(line);
		if (label !== null) {
			out.push(`{comment: ${label}}`);
			continue;
		}

		if (isChordLine(line)) {
			const chords = positionedChords(line);
			const next = lines[i + 1];
			const nextIsLyric =
				next !== undefined &&
				next.trim() !== '' &&
				!isChordLine(next) &&
				sectionLabel(next) === null;

			if (nextIsLyric) {
				out.push(mergeChordLine(chords, next));
				i += 1;
			} else {
				out.push(chords.map((c) => `[${c.chord}]`).join(' '));
			}
			continue;
		}

		out.push(line);
	}

	return out.join('\n');
}
