// The ribbon styles selected words in a story and a table cell, and shows mixed looks truthfully
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { DEFAULT_MARKS } from '../canvas/marksModel';
import { editorAction, editorState, type EditorInput } from './editorCommands';

const run = (text: string, extra = {}) => ({
	text,
	style: null,
	bold: false,
	italic: false,
	...extra,
});

const flowed = {
	type: 'text',
	id: 'fr_flow',
	box: {},
	locked: false,
	style: 'body',
	flow: { id: 'fl', index: 0 },
} as unknown as Frame;

const table = {
	type: 'table',
	id: 'fr_table',
	box: {},
	locked: false,
	cols: [1],
	head: 0,
	look: 'lined',
	rows: [[{ content: { inline: [run('one two\nthree', {})] } }]],
} as unknown as Frame;

const flows = [
	{
		id: 'fl',
		paragraphs: [
			{ inline: [run('A '), run('chapter', { bold: true, font: 'Zed Sans' })], look: null },
			{ inline: [run('The otter slept.', { font: 'Zed Sans' })], look: null },
		],
	},
] as never;

const input = (selected: Frame[], extra: Partial<EditorInput> = {}): EditorInput => ({
	spread: { id: 'sp', kind: 'spread', pages: [], frames: [], guides: [] } as unknown as Spread,
	spreadIndex: 1,
	spreadCount: 3,
	selected,
	layout: null,
	choices: [{ id: 'body', name: 'Body', size: 13 }],
	marks: DEFAULT_MARKS,
	snap: true,
	rulers: false,
	flows,
	...extra,
});

const story = { type: 'story', flow: 'fl' } as const;
const cell = { type: 'cell', frame: 'fr_table', row: 0, col: 0 } as const;

describe('words selected in a story', () => {
	const sel = (from: [number, number], to: [number, number]) => ({
		frame: 'fr_flow',
		start: 1,
		end: 2,
		target: story,
		from: { para: from[0], offset: from[1] },
		to: { para: to[0], offset: to[1] },
	});

	it('are styled through StyleText with the paragraph and place of the stretch', () => {
		const selection = sel([0, 2], [1, 3]);
		expect(editorAction('bold', undefined, input([flowed], { selection }))).toEqual({
			kind: 'intents',
			intents: [
				{
					type: 'StyleText',
					target: story,
					from: { para: 0, offset: 2 },
					to: { para: 1, offset: 3 },
					change: { bold: true },
				},
			],
		});
		expect(editorAction('italic', undefined, input([flowed], { selection }))).toMatchObject({
			intents: [{ type: 'StyleText', change: { italic: true } }],
		});
		expect(editorAction('colour', '#a23b2a', input([flowed], { selection }))).toMatchObject({
			intents: [{ type: 'StyleText', change: { colour: '#a23b2a' } }],
		});
		expect(editorAction('font', 'Zed Sans', input([flowed], { selection }))).toMatchObject({
			kind: 'font',
			intents: [{ type: 'StyleText', change: { font: 'Zed Sans' } }],
		});
	});

	it('turn bold off only when every character is bold, and show bold the same way', () => {
		const whole = input([flowed], { selection: sel([0, 2], [0, 9]) });
		expect(editorState(whole).active.bold).toBe(true);
		expect(editorAction('bold', undefined, whole)).toMatchObject({
			intents: [{ change: { bold: false } }],
		});
		const mixed = input([flowed], { selection: sel([0, 0], [0, 9]) });
		expect(editorState(mixed).active.bold).toBe(false);
		expect(editorAction('bold', undefined, mixed)).toMatchObject({
			intents: [{ change: { bold: true } }],
		});
	});

	it('show a font only when the words share one', () => {
		expect(editorState(input([flowed], { selection: sel([0, 2], [1, 5]) })).texts.font).toBe(
			'Zed Sans',
		);
		expect(editorState(input([flowed], { selection: sel([0, 0], [0, 9]) })).texts.font).toBe('');
		expect(editorState(input([flowed], { selection: sel([0, 2], [0, 5]) })).texts.font).toBe(
			'Zed Sans',
		);
	});

	it('leave a pending look to the caret: the look at the caret toggles', () => {
		const caret = { frame: 'fr_flow', offset: 5, target: story, pos: { para: 0, offset: 5 } };
		expect(editorAction('bold', undefined, input([flowed], { caret }))).toEqual({
			kind: 'pending',
			frame: 'fr_flow',
			change: { bold: false },
		});
		expect(editorState(input([flowed], { caret })).active.bold).toBe(true);
	});

	it('are left alone when the frame is locked or not the one selected', () => {
		const selection = sel([0, 0], [0, 3]);
		const locked = { ...flowed, locked: true } as Frame;
		expect(editorAction('bold', undefined, input([locked], { selection }))).not.toMatchObject({
			intents: [{ type: 'StyleText' }],
		});
		expect(editorAction('bold', undefined, input([], { selection }))).not.toMatchObject({
			intents: [{ type: 'StyleText' }],
		});
	});
});

describe('words selected in a table cell', () => {
	const selection = {
		frame: 'fr_table',
		start: 4,
		end: 11,
		target: cell,
		from: { para: 0, offset: 4 },
		to: { para: 1, offset: 2 },
	};

	it('make the controls available and style across the cell paragraphs', () => {
		const state = editorState(input([table], { selection }));
		for (const id of ['bold', 'italic', 'font', 'colour'])
			expect(state.available.has(id)).toBe(true);
		expect(editorAction('bold', undefined, input([table], { selection }))).toEqual({
			kind: 'intents',
			intents: [
				{
					type: 'StyleText',
					target: cell,
					from: { para: 0, offset: 4 },
					to: { para: 1, offset: 2 },
					change: { bold: true },
				},
			],
		});
		expect(editorAction('colour', '#a23b2a', input([table], { selection }))).toMatchObject({
			intents: [{ type: 'StyleText', target: cell, change: { colour: '#a23b2a' } }],
		});
	});

	it('set a look for the next words at a caret in the cell', () => {
		const caret = { frame: 'fr_table', offset: 3, target: cell, pos: { para: 0, offset: 3 } };
		expect(editorAction('italic', undefined, input([table], { caret }))).toEqual({
			kind: 'pending',
			frame: 'fr_table',
			change: { italic: true },
		});
	});
});

describe('words selected in a frame', () => {
	it('show bold only when the whole selection is bold', () => {
		const frame = {
			type: 'text',
			id: 't',
			box: {},
			locked: false,
			style: 'body',
			content: { inline: [run('Hello '), run('brave', { bold: true }), run(' world')] },
		} as unknown as Frame;
		const at = (start: number, end: number) =>
			editorState(input([frame], { selection: { frame: 't', start, end } })).active.bold;
		expect(at(6, 11)).toBe(true);
		expect(at(3, 8)).toBe(false);
	});
});

describe('a font chosen with the caret alone in the words', () => {
	const caret = { frame: 'fr_flow', offset: 5, target: story, pos: { para: 0, offset: 5 } };

	it('is for the words typed next, and an installed font is kept with the book', () => {
		const fonts = [
			{ family: 'Zed Sans', origin: 'system' },
			{ family: 'Inter', origin: 'bundled' },
		] as never;
		expect(editorAction('font', 'Zed Sans', input([flowed], { caret, fonts }))).toEqual({
			kind: 'pending',
			frame: 'fr_flow',
			change: { font: 'Zed Sans' },
			keep: 'Zed Sans',
		});
		expect(editorAction('font', 'Inter', input([flowed], { caret, fonts }))).toMatchObject({
			kind: 'pending',
			keep: null,
		});
	});

	it('shows the chosen font until the caret moves, then the font of the words there', () => {
		expect(editorState(input([flowed], { caret })).texts.font).toBe('Zed Sans');
		const pending = { font: 'Inter' };
		expect(editorState(input([flowed], { caret, pending })).texts.font).toBe('Inter');
	});

	it('works in a table cell and in a frame of its own', () => {
		const inCell = { frame: 'fr_table', offset: 2, target: cell, pos: { para: 0, offset: 2 } };
		expect(editorAction('font', 'Inter', input([table], { caret: inCell }))).toMatchObject({
			kind: 'pending',
			frame: 'fr_table',
			change: { font: 'Inter' },
		});
		const own = {
			type: 'text',
			id: 't',
			box: {},
			locked: false,
			style: 'body',
			content: { inline: [run('Hello')] },
		} as unknown as Frame;
		expect(
			editorAction('font', 'Inter', input([own], { caret: { frame: 't', offset: 3 } })),
		).toMatchObject({ kind: 'pending', frame: 't', change: { font: 'Inter' } });
	});
});

describe('underline and strike', () => {
	const own = {
		type: 'text',
		id: 'u',
		box: {},
		locked: false,
		style: 'body',
		content: {
			inline: [
				run('Hello '),
				run('brave', { underline: true, strike: true }),
				run(' world', { underline: true }),
			],
		},
	} as unknown as Frame;
	const range = (start: number, end: number) => ({ frame: 'u', start, end });

	it('are available with the words selected and pressed only when every character has them', () => {
		const at = (start: number, end: number) =>
			editorState(input([own], { selection: range(start, end) }));
		expect(at(6, 11).available.has('underline')).toBe(true);
		expect(at(6, 11).available.has('strike')).toBe(true);
		expect(at(6, 11).active.underline).toBe(true);
		expect(at(6, 11).active.strike).toBe(true);
		// Across the struck word and the next one, every character is underlined but not all struck.
		expect(at(6, 17).active.underline).toBe(true);
		expect(at(6, 17).active.strike).toBe(false);
		expect(at(0, 8).active.underline).toBe(false);
	});

	it('style the selected stretch, turning off when it is all underlined', () => {
		expect(editorAction('underline', undefined, input([own], { selection: range(0, 5) }))).toEqual({
			kind: 'intents',
			intents: [{ type: 'StyleRange', frame: 'u', start: 0, end: 5, change: { underline: true } }],
		});
		expect(editorAction('underline', undefined, input([own], { selection: range(6, 17) }))).toEqual(
			{
				kind: 'intents',
				intents: [
					{ type: 'StyleRange', frame: 'u', start: 6, end: 17, change: { underline: false } },
				],
			},
		);
		expect(
			editorAction('strike', undefined, input([own], { selection: range(6, 17) })),
		).toMatchObject({
			intents: [{ change: { strike: true } }],
		});
	});

	it('act on a story and set a pending look at a caret, which the caret then shows', () => {
		const selection = {
			frame: 'fr_flow',
			start: 1,
			end: 2,
			target: story,
			from: { para: 0, offset: 2 },
			to: { para: 1, offset: 3 },
		};
		expect(editorAction('underline', undefined, input([flowed], { selection }))).toMatchObject({
			intents: [{ type: 'StyleText', target: story, change: { underline: true } }],
		});
		const caret = { frame: 'fr_flow', offset: 5, target: story, pos: { para: 0, offset: 5 } };
		expect(editorAction('underline', undefined, input([flowed], { caret }))).toEqual({
			kind: 'pending',
			frame: 'fr_flow',
			change: { underline: true },
		});
		const pending = { underline: true };
		expect(editorState(input([flowed], { caret, pending })).active.underline).toBe(true);
		// Pressing it again takes the pending look back.
		expect(editorAction('underline', undefined, input([flowed], { caret, pending }))).toMatchObject(
			{
				change: { underline: false },
			},
		);
	});

	it("go on all of a selected frame's words when no editor is open", () => {
		expect(editorState(input([own])).available.has('underline')).toBe(true);
		expect(editorAction('strike', undefined, input([own]))).toEqual({
			kind: 'intents',
			intents: [{ type: 'StyleRange', frame: 'u', start: 0, end: 17, change: { strike: true } }],
		});
	});
});

describe('size and letter spacing', () => {
	const own = {
		type: 'text',
		id: 'z',
		box: {},
		locked: false,
		style: 'body',
		content: {
			inline: [
				run('Hello '),
				run('brave', { size: 20, tracking: 30 }),
				run(' world', { tracking: 30 }),
			],
		},
	} as unknown as Frame;
	const range = (start: number, end: number) => ({ frame: 'z', start, end });
	const at = (start: number, end: number) =>
		editorState(input([own], { selection: range(start, end) }));

	it('show the size of the words, or the first one with the control marked mixed', () => {
		expect(at(6, 11).values.size).toBe(20);
		expect(at(6, 11).mixed?.size).toBe(false);
		// The style's size is 13 in these choices.
		expect(at(0, 5).values.size).toBe(13);
		expect(at(0, 11).values.size).toBe(13);
		expect(at(0, 11).mixed?.size).toBe(true);
	});

	it('size the selected stretch, and the next words at a caret, in points', () => {
		expect(editorAction('size', 18, input([own], { selection: range(0, 5) }))).toEqual({
			kind: 'intents',
			intents: [{ type: 'StyleRange', frame: 'z', start: 0, end: 5, change: { size: 18 } }],
		});
		const caret = { frame: 'z', offset: 5 };
		expect(editorAction('size', 18, input([own], { caret }))).toEqual({
			kind: 'pending',
			frame: 'z',
			change: { size: 18 },
		});
		expect(editorState(input([own], { caret, pending: { size: 18 } })).values.size).toBe(18);
		// With no editor open the frame is sized whole, as before.
		expect(editorAction('size', 18, input([own]))).toMatchObject({
			intents: [{ type: 'OverrideStyle', frame: 'z' }],
		});
	});

	it('offer the spacings with a tick on the one every character shares, and none when they differ', () => {
		const tick = (state: ReturnType<typeof editorState>) =>
			(state.options.tracking ?? []).filter(([, label]) => label.startsWith('✓'));
		expect(tick(at(6, 17))).toEqual([['30', '✓ Loose']]);
		expect(tick(at(0, 5))).toEqual([['style', "✓ The style's"]]);
		expect(tick(at(0, 11))).toEqual([]);
		expect(at(0, 5).available.has('tracking')).toBe(true);
		expect((at(0, 5).options.tracking ?? []).map(([value]) => value)).toEqual([
			'-20',
			'0',
			'30',
			'80',
			'style',
		]);
	});

	it('set and take away the spacing of the selection, or of the next words', () => {
		expect(editorAction('tracking', '80', input([own], { selection: range(0, 5) }))).toEqual({
			kind: 'intents',
			intents: [{ type: 'StyleRange', frame: 'z', start: 0, end: 5, change: { tracking: 80 } }],
		});
		expect(
			editorAction('tracking', 'style', input([own], { selection: range(6, 11) })),
		).toMatchObject({
			intents: [{ change: { clearTracking: true } }],
		});
		expect(editorAction('tracking', '0', input([own], { selection: range(6, 11) }))).toMatchObject({
			intents: [{ change: { tracking: 0 } }],
		});
		expect(
			editorAction('tracking', '30', input([own], { caret: { frame: 'z', offset: 2 } })),
		).toEqual({ kind: 'pending', frame: 'z', change: { tracking: 30, clearTracking: false } });
		// A frame selected with no editor open has its words all spaced.
		expect(editorAction('tracking', '80', input([own]))).toMatchObject({
			intents: [{ type: 'StyleRange', start: 0, end: 17, change: { tracking: 80 } }],
		});
		expect(editorAction('tracking', 'nonsense', input([own]))).toBeNull();
	});

	it('work in a story and show the pending spacing', () => {
		const selection = {
			frame: 'fr_flow',
			start: 1,
			end: 2,
			target: story,
			from: { para: 0, offset: 0 },
			to: { para: 1, offset: 3 },
		};
		expect(editorAction('size', 9, input([flowed], { selection }))).toMatchObject({
			intents: [{ type: 'StyleText', target: story, change: { size: 9 } }],
		});
		const caret = { frame: 'fr_flow', offset: 5, target: story, pos: { para: 0, offset: 5 } };
		const state = editorState(input([flowed], { caret, pending: { tracking: 80 } }));
		expect(state.options.tracking?.find(([v]) => v === '80')?.[1]).toBe('✓ Very loose');
	});
});
