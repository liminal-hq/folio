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
