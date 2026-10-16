/** Public chord-engine types (frozen API — do not rename or reshape). */

export type Accidental = 'sharp' | 'flat';

export interface ChordSegment {
	chord: string | null;
	lyric: string;
}

export type ChartLine =
	{ kind: 'lyric'; text: string } | { kind: 'chord'; segments: ChordSegment[] } | { kind: 'empty' };

export interface ChartSection {
	label: string | null;
	lines: ChartLine[];
}

export interface Chart {
	meta: Record<string, string>;
	sections: ChartSection[];
}
