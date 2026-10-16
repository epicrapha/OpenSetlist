import { describe, expect, it } from 'vitest';
import {
	capo,
	detectKey,
	fromPlainText,
	parseChordPro,
	toChordPro,
	transpose,
	transposeKey,
	type Chart
} from './index';

const BLESSED = [
	'{title: Blessed Be Your Name}',
	'{artist: Matt Redman}',
	'{key: A}',
	'{tempo: 132}',
	'{time: 4/4}',
	'{ccli: 3798438}',
	'',
	'{comment: Verse 1}',
	'[A]Blessed be Your [E]name',
	'[F#m]In the land that is [D]plentiful',
	'',
	'{start_of_chorus}',
	'[D]Blessed be Your [A]name',
	'[E]Blessed be Your [F#m]name',
	'{end_of_chorus}'
].join('\n');

const CORNERSTONE = [
	'{title: Cornerstone}',
	'{key: C}',
	'',
	'{sov}',
	'[C]My hope is built on [G/B]nothing less',
	"[Am]Than Jesus' blood and [F]righteousness",
	'{eov}',
	'{soc}',
	'[C/E]Christ alone, [G]cornerstone',
	'{eoc}'
].join('\n');

const chordsOf = (chart: Chart): string[] =>
	chart.sections.flatMap((s) =>
		s.lines.flatMap((l) =>
			l.kind === 'chord' ? l.segments.flatMap((seg) => (seg.chord ? [seg.chord] : [])) : []
		)
	);

describe('parseChordPro', () => {
	it('reads directives into meta', () => {
		const chart = parseChordPro(BLESSED);
		expect(chart.meta).toMatchObject({
			title: 'Blessed Be Your Name',
			artist: 'Matt Redman',
			key: 'A',
			tempo: '132',
			time: '4/4',
			ccli: '3798438'
		});
	});

	it('opens sections on comment and structural directives', () => {
		const chart = parseChordPro(BLESSED);
		expect(chart.sections.map((s) => s.label)).toEqual(['Verse 1', 'Chorus']);
	});

	it('splits inline chords into chord/lyric segments', () => {
		const verse = parseChordPro(BLESSED).sections[0];
		expect(verse.lines[0]).toEqual({
			kind: 'chord',
			segments: [
				{ chord: 'A', lyric: 'Blessed be Your ' },
				{ chord: 'E', lyric: 'name' }
			]
		});
		expect(verse.lines[1]).toEqual({
			kind: 'chord',
			segments: [
				{ chord: 'F#m', lyric: 'In the land that is ' },
				{ chord: 'D', lyric: 'plentiful' }
			]
		});
	});

	it('ignores blank separators before the first section', () => {
		const chart = parseChordPro(BLESSED);
		expect(chart.sections[0].lines[0].kind).toBe('chord');
	});

	it('recognizes slash chords and extensions', () => {
		const chart = parseChordPro(CORNERSTONE);
		expect(chordsOf(chart)).toEqual(['C', 'G/B', 'Am', 'F', 'C/E', 'G']);
	});

	it('treats N.C. as a valid, untransposable chord token', () => {
		const chart = parseChordPro('{comment: X}\n[N.C.]No chord here');
		expect(chart.sections[0].lines[0]).toEqual({
			kind: 'chord',
			segments: [{ chord: 'N.C.', lyric: 'No chord here' }]
		});
		expect(chordsOf(transpose(chart, 3))).toEqual(['N.C.']);
	});
});

describe('round trip', () => {
	for (const [name, source] of [
		['Blessed', BLESSED],
		['Cornerstone', CORNERSTONE]
	] as const) {
		it(`${name}: toChordPro(parse(x)) re-parses identically`, () => {
			const once = parseChordPro(source);
			expect(parseChordPro(toChordPro(once))).toEqual(once);
		});

		it(`${name}: serialization is idempotent`, () => {
			const once = toChordPro(parseChordPro(source));
			expect(toChordPro(parseChordPro(once))).toBe(once);
		});
	}
});

describe('transpose', () => {
	it('shifts chords and key down two semitones (identity respell)', () => {
		const out = transpose(parseChordPro(BLESSED), -2);
		expect(out.meta.key).toBe('G');
		expect(chordsOf(out)).toEqual(['G', 'D', 'Em', 'C', 'C', 'G', 'D', 'Em']);
		expect(chordsOf(out)).not.toContain('F#m');
	});

	it('is an identity clone at zero semitones', () => {
		const chart = parseChordPro(BLESSED);
		const out = transpose(chart, 0);
		expect(out).toEqual(chart);
		expect(out).not.toBe(chart);
	});

	it('does not mutate its input', () => {
		const chart = parseChordPro(BLESSED);
		const snapshot = JSON.stringify(chart);
		transpose(chart, 5);
		expect(JSON.stringify(chart)).toBe(snapshot);
	});

	it('spells diatonic notes by the target key scale', () => {
		const out = transpose(parseChordPro(CORNERSTONE), 1);
		expect(out.meta.key).toBe('Db');
		expect(chordsOf(out)).toEqual(['Db', 'Ab/C', 'Bbm', 'Gb', 'Db/F', 'Ab']);
	});

	it('honors an explicit sharp preference', () => {
		const out = transpose(parseChordPro(BLESSED), 1, 'sharp');
		expect(out.meta.key).toBe('A#');
		expect(chordsOf(out)).toEqual(['A#', 'F', 'Gm', 'D#', 'D#', 'A#', 'F', 'Gm']);
	});

	it('honors an explicit flat preference', () => {
		const out = transpose(parseChordPro(BLESSED), 1, 'flat');
		expect(out.meta.key).toBe('Bb');
		expect(chordsOf(out)).toEqual(['Bb', 'F', 'Gm', 'Eb', 'Eb', 'Bb', 'F', 'Gm']);
	});

	it('never transposes N.C.', () => {
		const chart = parseChordPro('{comment: X}\n[N.C.]');
		expect(chordsOf(transpose(chart, 6, 'sharp'))).toEqual(['N.C.']);
	});
});

describe('transposeKey', () => {
	it('uses the circle-of-fifths convention by default', () => {
		expect(transposeKey('C', 1)).toBe('Db');
		expect(transposeKey('A', -2)).toBe('G');
		expect(transposeKey('F', -1)).toBe('E');
		expect(transposeKey('Bb', 1)).toBe('B');
		expect(transposeKey('Am', 2)).toBe('Bm');
	});

	it('respects an explicit preference and no-ops at zero', () => {
		expect(transposeKey('C', 1, 'sharp')).toBe('C#');
		expect(transposeKey('C', 12)).toBe('C');
		expect(transposeKey('A minor', 0)).toBe('A minor');
	});
});

describe('capo', () => {
	it('reports sounding key and shape key', () => {
		const result = capo(parseChordPro(BLESSED), 2);
		expect(result.soundingKey).toBe('A');
		expect(result.shapeKey).toBe('G');
		expect(result.shapes.meta.key).toBe('G');
		expect(chordsOf(result.shapes)).toEqual(['G', 'D', 'Em', 'C', 'C', 'G', 'D', 'Em']);
	});

	it('spells shapes for the transposed key', () => {
		const result = capo(parseChordPro(CORNERSTONE), 2);
		expect(result.soundingKey).toBe('C');
		expect(result.shapeKey).toBe('Bb');
		expect(chordsOf(result.shapes)).toEqual(['Bb', 'F/A', 'Gm', 'Eb', 'Bb/D', 'F']);
	});
});

describe('detectKey', () => {
	it('prefers the explicit meta key (normalized)', () => {
		expect(detectKey(parseChordPro(BLESSED))).toBe('A');
		expect(detectKey(parseChordPro('{key: A minor}\n[Am]'))).toBe('Am');
	});

	it('infers a minor key from a chord chart', () => {
		expect(detectKey(parseChordPro('{comment: Intro}\n[Em] [C] [G] [D]'))).toBe('Em');
	});

	it('returns null when there is nothing to go on', () => {
		expect(detectKey(parseChordPro('just some words'))).toBeNull();
	});
});

describe('fromPlainText', () => {
	it('merges a chord row onto the lyric row below it', () => {
		const source = [
			'Blessed Be Your Name',
			'',
			'Verse 1:',
			'A' + ' '.repeat(15) + 'E',
			'Blessed be Your name',
			'F#m' + ' '.repeat(17) + 'D',
			'In the land that is plentiful',
			'',
			'Chorus:',
			'D' + ' '.repeat(15) + 'A',
			'Blessed be Your name'
		].join('\n');

		const expected = [
			'Blessed Be Your Name',
			'',
			'{comment: Verse 1}',
			'[A]Blessed be Your [E]name',
			'[F#m]In the land that is [D]plentiful',
			'',
			'{comment: Chorus}',
			'[D]Blessed be Your [A]name'
		].join('\n');

		expect(fromPlainText(source)).toBe(expected);
	});

	it('keeps chord-only rows and strips bar lines', () => {
		expect(fromPlainText('| C | G |')).toBe('[C] [G]');
	});

	it('output parses back into sections and chords', () => {
		const chart = parseChordPro(fromPlainText('Chorus:\n[A]La [E]la'));
		expect(chart.sections[0].label).toBe('Chorus');
		expect(chordsOf(chart)).toEqual(['A', 'E']);
	});
});
