// Tests for which ribbon controls are live for a selection and what they ask for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { DEFAULT_MARKS } from '../canvas/marksModel';
import { editorAction, editorState, type EditorInput } from './editorCommands';

const spread = (kind = 'spread') =>
	({ id: 'sp', kind, pages: [], frames: [], guides: [] }) as unknown as Spread;
const text = (extra = {}) =>
	({ type: 'text', id: 't', box: {}, locked: false, style: 'body', ...extra }) as unknown as Frame;
const image = (extra = {}) =>
	({
		type: 'image',
		id: 'i',
		box: {},
		locked: false,
		image: { asset: null, fit: 'fill' },
		...extra,
	}) as unknown as Frame;

const input = (selected: Frame[] = [], extra: Partial<EditorInput> = {}): EditorInput => ({
	spread: spread(),
	spreadIndex: 1,
	spreadCount: 3,
	selected,
	layout: null,
	choices: [{ id: 'body', name: 'Body', size: 13 }],
	marks: DEFAULT_MARKS,
	snap: true,
	rulers: false,
	...extra,
});

const shape = (extra = {}) =>
	({
		type: 'shape',
		id: 's',
		box: {},
		locked: false,
		rotation: 0,
		shape: 'rect',
		...extra,
	}) as unknown as Frame;

describe('the spelling toggle', () => {
	it('is always available and asks to be switched', () => {
		expect(editorState(input()).available.has('spell')).toBe(true);
		expect(editorAction('spell', undefined, input())).toEqual({ kind: 'spelling' });
	});
});

describe('adding a table', () => {
	it('is always available and asks for a table to be added', () => {
		expect(editorState(input()).available.has('table')).toBe(true);
		expect(editorAction('table', undefined, input())).toEqual({ kind: 'add', shape: 'table' });
	});
});

describe('editing a table', () => {
	const table = (extra = {}) =>
		({
			type: 'table',
			id: 'tb',
			box: {},
			locked: false,
			rotation: 0,
			head: 1,
			look: 'lined',
			cols: [0.5, 0.5],
			rows: [
				[{}, {}],
				[{}, {}],
			],
			...extra,
		}) as unknown as Frame;
	const edit = (id: string, frame = table(), extra: Partial<EditorInput> = {}) =>
		editorAction(id, undefined, input([frame], extra));

	it('shows the Table tab and its controls for one table, with the look and header on', () => {
		const s = editorState(input([table()]));
		expect(s.contextual).toEqual(['table']);
		for (const id of ['rowAdd', 'rowDel', 'colAdd', 'colDel', 'headToggle', 'tableLined']) {
			expect(s.available.has(id), id).toBe(true);
		}
		expect(s.active).toMatchObject({ headToggle: true, tableLined: true, tableGrid: false });
	});

	it('keeps the last row and column, and is off for a locked table', () => {
		const one = editorState(input([table({ rows: [[{}]], cols: [1] })]));
		expect(one.available.has('rowDel')).toBe(false);
		expect(one.available.has('colDel')).toBe(false);
		expect(editorState(input([table({ locked: true })])).available.has('rowAdd')).toBe(false);
		expect(edit('rowAdd', table({ locked: true }))).toBeNull();
	});

	it('adds and removes at the end until a cell says where', () => {
		const intent = (action: ReturnType<typeof edit>) =>
			action && action.kind === 'intents' ? action.intents[0] : null;
		expect(intent(edit('rowAdd'))).toEqual({
			type: 'EditTable',
			frame: 'tb',
			edit: { type: 'addRow', at: 2 },
		});
		expect(intent(edit('colDel'))).toMatchObject({ edit: { type: 'removeColumn', at: 1 } });
		const here = { cell: { row: 0, col: 1 } };
		expect(intent(edit('rowAdd', table(), here))).toMatchObject({ edit: { at: 0 } });
		expect(intent(edit('colAdd', table(), here))).toMatchObject({ edit: { at: 1 } });
		expect(intent(edit('rowDel', table(), here))).toMatchObject({ edit: { at: 0 } });
	});

	it('toggles the header and picks a look', () => {
		const intent = (action: ReturnType<typeof edit>) =>
			action && action.kind === 'intents' ? action.intents[0] : null;
		expect(intent(edit('headToggle'))).toMatchObject({ edit: { type: 'setHead', head: 0 } });
		expect(intent(edit('headToggle', table({ head: 0 })))).toMatchObject({
			edit: { type: 'setHead', head: 1 },
		});
		expect(intent(edit('tableRecipe'))).toMatchObject({
			edit: { type: 'setLook', look: 'recipe' },
		});
		expect(intent(edit('tableGrid'))).toMatchObject({ edit: { look: 'grid' } });
	});
});

describe('turning a frame', () => {
	it('offers the turn buttons for one free frame only', () => {
		expect(editorState(input([shape()])).available.has('rotL')).toBe(true);
		expect(editorState(input([image()])).available.has('rotR')).toBe(true);
		expect(editorState(input([text()])).available.has('rotL')).toBe(true);
		expect(editorState(input([shape({ locked: true })])).available.has('rotR')).toBe(false);
		expect(editorState(input([shape(), image({ id: 'j' })])).available.has('rotL')).toBe(false);
	});

	it('cannot stretch a turned picture to the bleed line', () => {
		const turnedPicture = image({ rotation: 90 });
		expect(editorState(input([turnedPicture])).available.has('toBleed')).toBe(false);
		expect(editorAction('toBleed', undefined, input([turnedPicture]))).toBeNull();
	});

	it('turns a quarter either way from where the shape is now', () => {
		const left = editorAction('rotL', undefined, input([shape({ rotation: 30 })]));
		expect(left).toEqual({
			kind: 'intents',
			intents: [{ type: 'RotateFrame', on: { spread: 'sp' }, frame: 's', degrees: -60 }],
		});
		const right = editorAction('rotR', undefined, input([shape({ rotation: 30 })]));
		expect(right).toMatchObject({ intents: [{ degrees: 120 }] });
		expect(editorAction('rotR', undefined, input([image({ locked: true })]))).toBeNull();
		expect(editorAction('rotR', undefined, input([image({ rotation: 0 })]))).toMatchObject({
			intents: [{ type: 'RotateFrame', degrees: 90 }],
		});
	});
});

describe('editorState', () => {
	it('lets the page controls work with nothing selected, and not the text ones', () => {
		const s = editorState(input());
		for (const id of ['text', 'image', 'addPage', 'dupPage', 'delPage', 'margins', 'zoomIn']) {
			expect(s.available.has(id)).toBe(true);
		}
		for (const id of ['bold', 'alignL', 'size', 'front', 'lock', 'replace']) {
			expect(s.available.has(id)).toBe(false);
		}
		expect(s.active).toMatchObject({ margins: true, bleed: true, grid: false, snap: true });
	});

	it('keeps the cover from being copied or removed, and the last spread from being removed', () => {
		const cover = editorState(input([], { spread: spread('cover') }));
		expect(cover.available.has('dupPage')).toBe(false);
		expect(cover.available.has('delPage')).toBe(false);
		expect(editorState(input([], { spreadCount: 1 })).available.has('delPage')).toBe(false);
	});

	it('offers the master pages on any spread, and asks to edit them with the master button', () => {
		const plain = input();
		expect(editorState(plain).available.has('master')).toBe(true);
		expect(editorAction('master', undefined, plain)).toEqual({ kind: 'editMaster' });
	});

	it('offers the page background on a spread, and asks for its dialog', () => {
		expect(editorState(input()).available.has('pageBg')).toBe(true);
		expect(editorState(input([], { master: true })).available.has('pageBg')).toBe(false);
		expect(editorAction('pageBg', undefined, input())).toEqual({ kind: 'pageBackground' });
	});

	it('keeps page operations and guide clearing for the spreads, and not for a master page', () => {
		const onMaster = editorState(input([], { master: true }));
		expect(onMaster.active.master).toBe(true);
		expect(onMaster.available.has('master')).toBe(true);
		for (const id of ['addPage', 'dupPage', 'delPage', 'rgClear']) {
			expect(onMaster.available.has(id)).toBe(false);
		}
	});

	it('turns on the text controls for a text frame and shows how it is set', () => {
		const s = editorState(
			input([text({ overrides: { size: 18, align: 'center', bold: true, italic: null } })]),
		);
		expect(s.available.has('bold')).toBe(true);
		expect(s.active).toMatchObject({ bold: true, italic: false, alignC: true, alignL: false });
		expect(s.values.size).toBe(18);
		expect(s.contextual).toEqual([]);
	});

	it('shows the Picture tab and its controls for one picture', () => {
		const s = editorState(input([image({ image: { asset: 'a', fit: 'fit' } })]));
		expect(s.contextual).toEqual(['picture']);
		expect(s.available.has('toBleed')).toBe(true);
		expect(s.active.fitFill).toBe(true);
		expect(editorState(input([image(), image({ id: 'j' })])).contextual).toEqual([]);
	});

	it('does not offer to arrange or restyle locked frames, but does offer to unlock them', () => {
		const s = editorState(input([text({ locked: true })]));
		expect(s.available.has('front')).toBe(false);
		expect(s.available.has('bold')).toBe(false);
		expect(s.available.has('lock')).toBe(true);
		expect(s.active.lock).toBe(true);
	});
});

describe('editorAction', () => {
	it('adds frames and pages', () => {
		expect(editorAction('text', undefined, input())).toEqual({ kind: 'add', shape: 'text' });
		expect(editorAction('image', undefined, input())).toEqual({ kind: 'add', shape: 'picture' });
		expect(editorAction('ellipse', undefined, input())).toEqual({ kind: 'add', shape: 'ellipse' });
		expect(editorAction('dupPage', undefined, input())).toEqual({ kind: 'page', op: 'duplicate' });
	});

	it('flips the page helpers', () => {
		expect(editorAction('margins', undefined, input())).toEqual({
			kind: 'marks',
			marks: { margins: false },
		});
		expect(editorAction('grid', undefined, input())).toEqual({
			kind: 'marks',
			marks: { columns: true, baseline: true },
		});
	});

	it('toggles bold from the first selected frame for all of them', () => {
		const out = editorAction('bold', undefined, input([text(), text({ id: 'u' })]));
		expect(out).toMatchObject({ kind: 'intents' });
		const intents = (out as { intents: unknown[] }).intents;
		expect(intents).toHaveLength(2);
		expect(intents[0]).toEqual({
			type: 'OverrideStyle',
			frame: 't',
			overrides: { size: null, align: null, bold: true, italic: null },
		});
	});

	it('sets the alignment and the size within bounds', () => {
		expect(editorAction('alignR', undefined, input([text()]))).toMatchObject({
			intents: [{ overrides: { align: 'right' } }],
		});
		expect(editorAction('size', 500, input([text()]))).toMatchObject({
			intents: [{ overrides: { size: 200 } }],
		});
		expect(editorAction('size', 'big', input([text()]))).toBeNull();
	});

	it('reorders only unlocked frames and locks or unlocks them all together', () => {
		const sel = [text(), text({ id: 'u', locked: true })];
		expect(editorAction('front', undefined, input(sel))).toMatchObject({
			intents: [{ type: 'Reorder', frame: 't', to: 'front' }],
		});
		expect(editorAction('lock', undefined, input(sel))).toMatchObject({
			intents: [{ type: 'LockFrame', frame: 't', locked: true }],
		});
		expect(editorAction('lock', undefined, input([text({ locked: true })]))).toMatchObject({
			intents: [{ type: 'LockFrame', frame: 't', locked: false }],
		});
	});

	it('acts on one unlocked picture, and toggles how it fits', () => {
		expect(editorAction('replace', undefined, input([image()]))).toEqual({
			kind: 'choosePicture',
			frame: 'i',
		});
		expect(editorAction('fitFill', undefined, input([image()]))).toEqual({
			kind: 'fit',
			frame: 'i',
			fit: 'fit',
		});
		expect(editorAction('toBleed', undefined, input([image({ locked: true })]))).toBeNull();
		expect(editorAction('replace', undefined, input([text()]))).toBeNull();
	});

	it('leaves other controls alone', () => {
		expect(editorAction('undo', undefined, input())).toBeNull();
	});
});

describe('the rulers and their guides', () => {
	it('lets the rulers and the guides be switched, and shows which are on', () => {
		const off = editorState(input());
		expect(off.available.has('rulers') && off.available.has('rguides')).toBe(true);
		expect(off.active.rulers).toBe(false);
		expect(off.active.rguides).toBe(true);
		const on = editorState(input([], { rulers: true, marks: { ...DEFAULT_MARKS, guides: false } }));
		expect(on.active.rulers).toBe(true);
		expect(on.active.rguides).toBe(false);
	});

	it('asks to toggle the rulers and the guides mark', () => {
		expect(editorAction('rulers', undefined, input())).toEqual({ kind: 'rulers' });
		expect(editorAction('rguides', undefined, input())).toEqual({
			kind: 'marks',
			marks: { guides: false },
		});
	});

	it('offers to clear guides only when the spread has any that are not locked', () => {
		const withGuides = (locked: boolean) =>
			input([], {
				spread: {
					...spread(),
					guides: [{ axis: 'x', pos: 10, locked }],
				} as unknown as Spread,
			});
		expect(editorState(input()).available.has('rgClear')).toBe(false);
		expect(editorState(withGuides(true)).available.has('rgClear')).toBe(false);
		expect(editorState(withGuides(false)).available.has('rgClear')).toBe(true);
		expect(editorAction('rgClear', undefined, withGuides(false))).toEqual({
			kind: 'intents',
			intents: [{ type: 'ClearGuides', spread: 'sp' }],
		});
	});
});

describe('the font list', () => {
	const fonts = [
		{ family: 'Literata', origin: 'bundled' },
		{ family: 'Zed Sans', origin: 'system' },
		{ family: 'Quill Mono', origin: 'book' },
	] as const;

	it("is live for words, offers the book's fonts and shows the font of the words", () => {
		expect(editorState(input([image()])).available.has('font')).toBe(false);
		const state = editorState(
			input([text({ overrides: { font: 'Zed Sans' } })], { fonts: [...fonts] }),
		);
		expect(state.available.has('font')).toBe(true);
		expect(state.texts.font).toBe('Zed Sans');
		expect(state.options.font).toEqual([
			['Literata', 'Literata'],
			['Zed Sans', 'Zed Sans'],
			['Quill Mono', 'Quill Mono'],
		]);
		expect(editorState(input([text()], { fonts: [...fonts] })).texts.font).toBe('');
	});

	it('sets the font of the words, and asks to keep an installed one with the book', () => {
		const frame = text({ overrides: { size: 12, align: null, bold: true, italic: null } });
		const action = editorAction('font', 'Zed Sans', input([frame], { fonts: [...fonts] }));
		expect(action).toMatchObject({ kind: 'font', keep: 'Zed Sans' });
		expect(action?.kind === 'font' && action.intents).toEqual([
			{
				type: 'OverrideStyle',
				frame: 't',
				overrides: { size: 12, align: null, bold: true, italic: null, font: 'Zed Sans' },
			},
		]);
	});

	it('keeps nothing for a bundled font or one the book already has, and clears with the empty choice', () => {
		const base = input([text()], { fonts: [...fonts] });
		expect(editorAction('font', 'Literata', base)).toMatchObject({ keep: null });
		expect(editorAction('font', 'Quill Mono', base)).toMatchObject({ keep: null });
		const had = input([text({ overrides: { font: 'Zed Sans' } })], { fonts: [...fonts] });
		const cleared = editorAction('font', '', had);
		expect(cleared).toMatchObject({ kind: 'font', keep: null });
		expect(cleared?.kind === 'font' && cleared.intents[0]).toMatchObject({
			overrides: { size: null, align: null, bold: null, italic: null },
		});
		expect(
			cleared?.kind === 'font' && 'font' in (cleared.intents[0] as { overrides: object }).overrides,
		).toBe(false);
	});
});

describe('the Check book button', () => {
	it('is always available and asks for the dialog', () => {
		expect(editorState(input()).available.has('preflight')).toBe(true);
		expect(editorAction('preflight', undefined, input())).toEqual({ kind: 'checkBook' });
	});
});

describe('the Export buttons', () => {
	it('are always available and ask for the dialog, quickly for Quick PDF', () => {
		const state = editorState(input());
		expect(state.available.has('export') && state.available.has('quickPdf')).toBe(true);
		expect(editorAction('export', undefined, input())).toEqual({
			kind: 'exportBook',
			quick: false,
		});
		expect(editorAction('quickPdf', undefined, input())).toEqual({
			kind: 'exportBook',
			quick: true,
		});
	});
});
