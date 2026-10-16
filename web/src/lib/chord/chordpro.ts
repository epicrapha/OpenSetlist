import { isChordToken } from './chords';
import type { Chart, ChartLine, ChartSection, ChordSegment } from './types';

// Structural directives -> default section label (value overrides the default).
const SECTION_START: Record<string, string> = {
	start_of_chorus: 'Chorus',
	soc: 'Chorus',
	chorus: 'Chorus',
	start_of_verse: 'Verse',
	sov: 'Verse',
	verse: 'Verse',
	start_of_bridge: 'Bridge',
	sob: 'Bridge',
	bridge: 'Bridge',
	start_of_tab: 'Tab',
	sot: 'Tab',
	tab: 'Tab',
	start_of_grid: 'Grid',
	sog: 'Grid',
	start_of_part: 'Part',
	sop: 'Part'
};

const STRUCTURAL_END = new Set([
	'end_of_chorus',
	'eoc',
	'end_of_verse',
	'eov',
	'end_of_bridge',
	'eob',
	'end_of_tab',
	'eot',
	'end_of_grid',
	'eog',
	'end_of_part',
	'eop'
]);

interface Directive {
	name: string;
	value?: string;
}

function parseDirective(line: string): Directive | null {
	const match = /^\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*(?::\s*([^}]*?))?\s*\}$/.exec(line);
	if (!match) return null;
	return {
		name: match[1].toLowerCase(),
		value: match[2] === undefined ? undefined : match[2].trim()
	};
}

/** Split a ChordPro lyric line into chord/lyric segments. Only valid chords split. */
function lineToSegments(line: string): ChordSegment[] {
	const segments: ChordSegment[] = [];
	let buffer = '';
	let chord: string | null = null;
	let index = 0;

	const flush = () => {
		if (buffer !== '' || chord !== null) {
			segments.push({ chord, lyric: buffer });
			buffer = '';
			chord = null;
		}
	};

	while (index < line.length) {
		if (line[index] === '[') {
			const end = line.indexOf(']', index);
			if (end !== -1) {
				const name = line.slice(index + 1, end);
				if (isChordToken(name)) {
					flush();
					chord = name;
					index = end + 1;
					continue;
				}
			}
		}
		buffer += line[index];
		index += 1;
	}

	flush();
	return segments;
}

function parseLine(line: string): ChartLine {
	const segments = lineToSegments(line);
	if (segments.some((segment) => segment.chord !== null)) return { kind: 'chord', segments };
	if (line.trim() === '') return { kind: 'empty' };
	return { kind: 'lyric', text: line };
}

export function parseChordPro(source: string): Chart {
	const meta: Record<string, string> = {};
	const sections: ChartSection[] = [];
	let current: ChartSection | null = null;

	const ensure = (): ChartSection => {
		if (!current) {
			current = { label: null, lines: [] };
			sections.push(current);
		}
		return current;
	};

	for (const raw of source.replace(/\r\n?/g, '\n').split('\n')) {
		const directive = parseDirective(raw.trim());
		if (directive) {
			const { name, value } = directive;

			const startLabel = SECTION_START[name];
			if (startLabel !== undefined) {
				current = { label: value || startLabel, lines: [] };
				sections.push(current);
				continue;
			}
			if (STRUCTURAL_END.has(name)) continue;
			if (name === 'comment' || name === 'c') {
				if (value === undefined) continue;
				current = { label: value, lines: [] };
				sections.push(current);
				continue;
			}
			if (value !== undefined) meta[name] = value;
			continue;
		}

		// Blank separators before any section are dropped; blanks inside a section are kept.
		if (raw.trim() === '' && current === null) continue;
		ensure().lines.push(parseLine(raw));
	}

	return { meta, sections };
}

export function toChordPro(chart: Chart): string {
	const out: string[] = [];

	for (const [name, value] of Object.entries(chart.meta)) out.push(`{${name}: ${value}}`);

	for (const section of chart.sections) {
		if (section.label !== null) out.push(`{comment: ${section.label}}`);
		for (const line of section.lines) {
			if (line.kind === 'empty') out.push('');
			else if (line.kind === 'lyric') out.push(line.text);
			else out.push(line.segments.map((s) => (s.chord ? `[${s.chord}]` : '') + s.lyric).join(''));
		}
	}

	return out.join('\n');
}
