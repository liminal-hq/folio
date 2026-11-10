// Tests for the Paragraph group: which paragraphs the controls act on, what they show and what they ask for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { DEFAULT_MARKS } from '../canvas/marksModel';
import { buildRibbonModel } from './ribbonModel';
import { DEFAULT_SETTINGS } from '../settings/fakeSettingsClient';
import { editorAction, editorState, type EditorInput } from './editorCommands';

const spread = { id: 'sp', kind: 'spread', pages: [], frames: [], guides: [] } as unknown as Spread;
const text = (extra = {}) =>
	({
		type: 'text',
		id: 't',
		box: {},
		locked: false,
		style: 'body',
		content: { inline: [{ text: 'One\nTwo\nThree' }] },
		...extra,
	}) as unknown as Frame;

const input = (selected: Frame[], extra: Partial<EditorInput> = {}): EditorInput => ({
	spread,
	spreadIndex: 1,
	spreadCount: 3,
	selected,
	layout: null,
	choices: [{ id: 'body', name: 'Body', size: 10 }],
	marks: DEFAULT_MARKS,
	snap: true,
	rulers: false,
	...extra,
});

const look = (target: object, from: number, to: number, set: object, clear: string[] = []) => ({
	type: 'SetParagraphLook',
	target,
	from,
	to,
	change: { set, clear },
});
const frameT = { type: 'frame', frame: 't' };

describe('the Paragraph group', () => {
	it('is live for a text frame and not for a picture or nothing', () => {
		const ids = [
			'lineSpacing',
			'spaceBefore',
			'spaceAfter',
			'indentLess',
			'indentMore',
			'firstIndent',
		];
		const live = editorState(input([text()])).available;
		for (const id of ids) expect(live.has(id), id).toBe(true);
		const none = editorState(input([])).available;
		for (const id of ids) expect(none.has(id), id).toBe(false);
		// Bullets, numbers, the drop cap and letter spacing need model work and stay off.
		for (const id of ['bullets', 'numbers', 'dropCap', 'tracking'])
			expect(live.has(id)).toBe(false);
	});

	it('spaces every paragraph of the frame when no editor is open', () => {
		expect(editorAction('spaceAfter', '12', input([text()]))).toEqual({
			kind: 'intents',
			intents: [look(frameT, 0, 2, { spaceAfter: 12 })],
		});
		expect(editorAction('spaceBefore', '4', input([text()]))).toMatchObject({
			intents: [{ change: { set: { spaceBefore: 4 } } }],
		});
		expect(editorAction('lineSpacing', '1.5', input([text()]))).toMatchObject({
			intents: [{ change: { set: { lineHeight: 1.5 } } }],
		});
	});

	it('gives the style its say again, and refuses what is not a number', () => {
		expect(editorAction('spaceAfter', 'style', input([text()]))).toMatchObject({
			intents: [{ change: { set: {}, clear: ['spaceAfter'] } }],
		});
		expect(editorAction('lineSpacing', 'style', input([text()]))).toMatchObject({
			intents: [{ change: { clear: ['lineHeight'] } }],
		});
		expect(editorAction('spaceAfter', 'lots', input([text()]))).toBeNull();
	});

	it('acts on only the paragraphs the caret or selection is in', () => {
		const paragraphs = { target: frameT as never, frame: 't', from: 1, to: 2, looks: [] };
		expect(editorAction('spaceAfter', '8', input([text()], { paragraphs }))).toEqual({
			kind: 'intents',
			intents: [look(frameT, 1, 2, { spaceAfter: 8 })],
		});
		// An editor open on a frame that is not selected is not the one the controls act on.
		const other = { ...paragraphs, frame: 'z' };
		expect(editorAction('spaceAfter', '8', input([text()], { paragraphs: other }))).toMatchObject({
			intents: [{ from: 0, to: 2 }],
		});
	});

	it('steps the indent of each paragraph from where it is, and not below nothing', () => {
		const content = {
			inline: [{ text: 'One\nTwo\nThree' }],
			looks: [{ indent: 36 }, { indent: 36 }, {}],
		};
		expect(editorAction('indentMore', undefined, input([text({ content })]))).toEqual({
			kind: 'intents',
			intents: [look(frameT, 0, 1, { indent: 54 }), look(frameT, 2, 2, { indent: 18 })],
		});
		expect(editorAction('indentLess', undefined, input([text({ content })]))).toEqual({
			kind: 'intents',
			intents: [look(frameT, 0, 1, { indent: 18 }), look(frameT, 2, 2, { indent: 0 })],
		});
	});

	it('turns the first-line indent on at 1.2 em of the size and off again', () => {
		expect(editorAction('firstIndent', undefined, input([text()]))).toMatchObject({
			intents: [{ change: { set: { firstIndent: 12 } } }],
		});
		const set = text({ content: { inline: [{ text: 'a' }], looks: [{ firstIndent: 12 }] } });
		expect(editorAction('firstIndent', undefined, input([set]))).toMatchObject({
			intents: [{ change: { set: { firstIndent: 0 } } }],
		});
		expect(editorState(input([set])).active.firstIndent).toBe(true);
	});

	it('shows the alignment and spacing the first paragraph has', () => {
		const frame = text({
			content: { inline: [{ text: 'a\nb' }], looks: [{ align: 'center', spaceAfter: 12 }] },
		});
		const state = editorState(input([frame]));
		expect(state.active).toMatchObject({ alignC: true, alignL: false });
		const after = state.options.spaceAfter!;
		expect(after.find(([value]) => value === '12')![1]).toMatch(/^✓/);
		expect(after.find(([value]) => value === '4')![1]).not.toMatch(/^✓/);
		expect(state.options.lineSpacing!.map(([v]) => v)).toEqual([
			'1',
			'1.15',
			'1.3',
			'1.5',
			'1.8',
			'2',
			'style',
		]);
	});

	it('changes the paragraphs of a story that a flowing frame shows', () => {
		const frame = text({ content: undefined, flow: { id: 'fl', index: 0 } });
		const layout = {
			frames: [{ frame: 't', lines: [3, 4, 4].map((paragraph) => ({ paragraph, runs: [] })) }],
		} as never;
		expect(editorAction('spaceAfter', '8', input([frame], { layout }))).toEqual({
			kind: 'intents',
			intents: [look({ type: 'story', flow: 'fl' }, 3, 4, { spaceAfter: 8 })],
		});
	});

	it('leaves a locked frame alone', () => {
		expect(editorState(input([text({ locked: true })])).available.has('spaceAfter')).toBe(false);
	});
});

describe('the ribbon model for the Paragraph group', () => {
	it('lists the spacing choices in the menus and explains each control', () => {
		const model = buildRibbonModel({
			settings: DEFAULT_SETTINGS,
			available: new Set(),
			editor: editorState(input([text()])),
		});
		const items = model.tabs.flatMap((tab) =>
			tab.groups.flatMap((g) => [...(g.items ?? []), ...(g.rows ?? []).flat(2)]),
		);
		const after = items.find((i) => i.id === 'spaceAfter')!;
		expect(after.disabled).toBe(false);
		expect(after.options!.map(([v]) => v)).toContain('18');
		expect(after.desc).toMatch(/Space below/);
		expect(items.find((i) => i.id === 'spaceBefore')).toBeDefined();
		expect(items.find((i) => i.id === 'bullets')!.disabled).toBe(true);
	});
});
