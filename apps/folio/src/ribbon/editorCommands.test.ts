// Tests for which ribbon controls are live for a selection and what they ask for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { DEFAULT_MARKS } from '../canvas/marksModel';
import { editorAction, editorState, fontOptions, type EditorInput } from './editorCommands';

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
			['Literata', 'Literata', 'In this book'],
			['Quill Mono', 'Quill Mono', 'In this book'],
			['Zed Sans', 'Zed Sans', 'Installed'],
		]);
		expect(editorState(input([text()], { fonts: [...fonts] })).texts.font).toBe('');
	});

	it('lists an installed font the book uses under the book, and keeps each group in order', () => {
		expect(fontOptions(fonts, ['Zed Sans'])).toEqual([
			['Literata', 'Literata', 'In this book'],
			['Zed Sans', 'Zed Sans', 'In this book'],
			['Quill Mono', 'Quill Mono', 'In this book'],
		]);
		const more = [...fonts, { family: 'Arial', origin: 'system' as const }];
		expect(fontOptions(more, []).map(([family, , group]) => `${group}: ${family}`)).toEqual([
			'In this book: Literata',
			'In this book: Quill Mono',
			'Installed: Zed Sans',
			'Installed: Arial',
		]);
		expect(
			editorState(input([text()], { fonts: [...fonts], fontsInUse: ['Zed Sans'] })).options
				.font?.[2],
		).toEqual(['Quill Mono', 'Quill Mono', 'In this book']);
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

describe('the Spread templates button', () => {
	it('is available on a page and asks for the gallery, but not while a master page is edited', () => {
		expect(editorState(input()).available.has('templates')).toBe(true);
		expect(editorAction('templates', undefined, input())).toEqual({ kind: 'templates' });
		expect(editorState(input([], { master: true })).available.has('templates')).toBe(false);
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

describe('the preview buttons', () => {
	it('are always available, show print as chosen to begin with and ask to switch', () => {
		const state = editorState(input());
		expect(['print', 'kindle', 'kobo'].every((id) => state.available.has(id))).toBe(true);
		expect(state.active.print).toBe(true);
		expect(editorState(input([], { preview: 'kobo' })).active).toMatchObject({ kobo: true });
		expect(editorAction('kindle', undefined, input())).toEqual({ kind: 'preview', mode: 'kindle' });
		expect(editorAction('print', undefined, input())).toEqual({ kind: 'preview', mode: 'print' });
	});
});

describe('words selected in the open editor', () => {
	const words = (inline: { text: string; bold?: boolean }[]) =>
		({
			type: 'text',
			id: 't',
			box: {},
			locked: false,
			rotation: 0,
			style: 'body',
			content: { inline: inline.map((r) => ({ bold: false, italic: false, ...r })) },
		}) as unknown as Frame;

	it('are what bold, italic and the font apply to, and not the whole frame', () => {
		const frame = words([{ text: 'Hello brave world' }]);
		const selection = { frame: 't', start: 6, end: 11 };
		expect(editorAction('bold', undefined, input([frame], { selection }))).toEqual({
			kind: 'intents',
			intents: [{ type: 'StyleRange', frame: 't', start: 6, end: 11, change: { bold: true } }],
		});
		expect(editorAction('font', 'Zed Sans', input([frame], { selection }))).toMatchObject({
			kind: 'font',
			intents: [
				{ type: 'StyleRange', frame: 't', start: 6, end: 11, change: { font: 'Zed Sans' } },
			],
		});
	});

	it('turn bold off when the whole stretch is already bold', () => {
		const frame = words([{ text: 'Hello ' }, { text: 'brave', bold: true }, { text: ' world' }]);
		const selection = { frame: 't', start: 6, end: 11 };
		expect(editorAction('bold', undefined, input([frame], { selection }))).toMatchObject({
			intents: [{ change: { bold: false } }],
		});
		// A stretch that is only partly bold is made bold.
		expect(
			editorAction(
				'bold',
				undefined,
				input([frame], { selection: { frame: 't', start: 3, end: 8 } }),
			),
		).toMatchObject({ intents: [{ change: { bold: true } }] });
	});

	it('are ignored when nothing is selected or the frame is not the one selected', () => {
		const frame = words([{ text: 'Hello' }]);
		for (const selection of [
			null,
			{ frame: 't', start: 2, end: 2 },
			{ frame: 'other', start: 0, end: 3 },
		]) {
			expect(editorAction('bold', undefined, input([frame], { selection }))).toMatchObject({
				intents: [{ type: 'OverrideStyle' }],
			});
		}
	});
});

describe('the text colour list', () => {
	const frame = {
		type: 'text',
		id: 't',
		box: {},
		locked: false,
		rotation: 0,
		style: 'body',
		content: { inline: [{ text: 'Hello brave world', bold: false, italic: false }] },
	} as unknown as Frame;

	it('offers colours once words are selected', () => {
		const state = editorState(input([frame]));
		expect(state.available.has('colour')).toBe(true);
		expect(state.options.colour?.[0]?.[0]).toBe('#1c1b20');
		expect(editorState(input([])).available.has('colour')).toBe(false);
	});

	it('colours the selected words, or all of them when none are selected', () => {
		expect(
			editorAction(
				'colour',
				'#a23b2a',
				input([frame], { selection: { frame: 't', start: 6, end: 11 } }),
			),
		).toEqual({
			kind: 'intents',
			intents: [
				{ type: 'StyleRange', frame: 't', start: 6, end: 11, change: { colour: '#a23b2a' } },
			],
		});
		expect(editorAction('colour', '#a23b2a', input([frame]))).toEqual({
			kind: 'intents',
			intents: [
				{ type: 'StyleRange', frame: 't', start: 0, end: 17, change: { colour: '#a23b2a' } },
			],
		});
		// Choosing "Colour from style" takes the colour away.
		expect(editorAction('colour', '', input([frame]))).toMatchObject({
			intents: [{ change: { colour: '' } }],
		});
	});
});

describe('the controls that were listed but never live', () => {
	it('offers the six ruler units and sets the one chosen', () => {
		const state = editorState(input());
		expect(state.available.has('rulerUnits')).toBe(true);
		expect(state.options.rulerUnits?.map(([unit]) => unit)).toEqual([
			'inches',
			'millimetres',
			'centimetres',
			'points',
			'picas',
			'pixels',
		]);
		expect(editorAction('rulerUnits', 'picas', input())).toEqual({
			kind: 'rulerUnit',
			unit: 'picas',
		});
		expect(editorAction('rulerUnits', 'cubits', input())).toBeNull();
	});

	it('toggles the baseline grid and the columns on their own, and shows them pressed', () => {
		const state = editorState(input([], { marks: { ...DEFAULT_MARKS, baseline: true } }));
		expect(state.available.has('baseline')).toBe(true);
		expect(state.available.has('columns')).toBe(true);
		expect(state.active.baseline).toBe(true);
		expect(state.active.columns).toBe(DEFAULT_MARKS.columns);
		expect(editorAction('baseline', true, input())).toEqual({
			kind: 'marks',
			marks: { baseline: !DEFAULT_MARKS.baseline },
		});
		expect(editorAction('columns', true, input())).toEqual({
			kind: 'marks',
			marks: { columns: !DEFAULT_MARKS.columns },
		});
	});

	it('adds a note box from the Insert tab and a shape from the Shape menu', () => {
		const state = editorState(input());
		expect(state.available.has('shape')).toBe(true);
		expect(editorAction('shape', undefined, input())).toEqual({ kind: 'add', shape: 'note' });
		expect(state.options.shapeMenu?.map(([id]) => id)).toEqual([
			'rect',
			'round',
			'ellipse',
			'line',
		]);
		expect(editorAction('shapeMenu', 'ellipse', input())).toEqual({
			kind: 'add',
			shape: 'ellipse',
		});
		expect(editorAction('shapeMenu', 'star', input())).toBeNull();
	});

	it('switches the toolbar between the ribbon and the classic toolbar', () => {
		const state = editorState(input());
		expect(state.available.has('toolbarLayout')).toBe(true);
		expect(state.options.toolbarLayout?.map(([id]) => id)).toEqual(['ribbon', 'classic']);
		expect(editorAction('toolbarLayout', 'classic', input())).toEqual({
			kind: 'toolbar',
			toolbar: 'classic',
		});
		expect(editorAction('toolbarLayout', 'rail', input())).toBeNull();
	});

	it('asks Jax only while Jax is switched on', () => {
		expect(editorState(input()).available.has('askJax')).toBe(false);
		expect(editorState(input([], { jax: true })).available.has('askJax')).toBe(true);
		expect(editorAction('askJax', undefined, input([], { jax: true }))).toEqual({ kind: 'jax' });
	});

	it('leaves the layouts that are not built, and the layout importer, unavailable', () => {
		const state = editorState(input([text()], { jax: true }));
		expect(state.available.has('sidebarLayout')).toBe(false);
		expect(state.available.has('importLayout')).toBe(false);
	});

	it('sets the paragraph style of the free text frames selected', () => {
		const choices = [
			{ id: 'body', name: 'Body', size: 13 },
			{ id: 'title', name: 'Title', size: 28 },
		];
		const state = editorState(input([text()], { choices }));
		expect(state.available.has('paragraphStyle')).toBe(true);
		expect(state.texts.paragraphStyle).toBe('body');
		expect(state.options.paragraphStyle).toEqual([
			['body', 'Body'],
			['title', 'Title'],
		]);
		expect(
			editorAction('paragraphStyle', 'title', input([text(), text({ id: 'u' })], { choices })),
		).toEqual({
			kind: 'intents',
			intents: [
				{ type: 'SetStyle', frame: 't', style: 'title' },
				{ type: 'SetStyle', frame: 'u', style: 'title' },
			],
		});
		expect(editorAction('paragraphStyle', '', input([text()], { choices }))).toBeNull();
		expect(editorState(input([], { choices })).available.has('paragraphStyle')).toBe(false);
		expect(
			editorState(input([text({ locked: true })], { choices })).available.has('paragraphStyle'),
		).toBe(false);
	});
});

describe('Align and Distribute', () => {
	const at = (id: string, x: number, y = 0) =>
		text({ id, box: { x, y, w: 20, h: 10 }, rotation: 0 });
	const pages = [{ x: 0, y: 0, w: 300, h: 400 }];
	const ids = ['alignLeftE', 'alignCenterE', 'alignRightE', 'alignTopE', 'alignMidE', 'alignBotE'];

	it('aligns two free frames, or one to its page, and spreads three', () => {
		const live = (selected: Frame[], extra = {}) => editorState(input(selected, extra)).available;
		expect(live([at('a', 0)], { pages }).has('alignLeftE')).toBe(true);
		expect(live([at('a', 0)]).has('alignLeftE')).toBe(false);
		expect(live([]).has('alignLeftE')).toBe(false);
		const two = live([at('a', 0), at('b', 50)]);
		for (const id of ids) expect(two.has(id), id).toBe(true);
		expect(two.has('distH')).toBe(false);
		const three = live([at('a', 0), at('b', 50), at('c', 90)]);
		expect(three.has('distH')).toBe(true);
		expect(three.has('distV')).toBe(true);
		// Locked frames and frames from a master do not count.
		const locked = live([at('a', 0), at('b', 50, 0), { ...at('c', 90), locked: true } as Frame]);
		expect(locked.has('distH')).toBe(false);
		expect(locked.has('alignLeftE')).toBe(true);
	});

	it('moves the frames in one batch, leaving the locked ones alone', () => {
		const a = at('a', 10);
		const b = at('b', 50);
		const lockedOne = { ...at('c', 200), locked: true } as Frame;
		expect(editorAction('alignLeftE', undefined, input([a, b, lockedOne]))).toEqual({
			kind: 'intents',
			intents: [
				{
					type: 'MoveFrames',
					on: { spread: 'sp' },
					moves: [{ frame: 'b', box: { x: 10, y: 0, w: 20, h: 10 } }],
				},
			],
		});
		expect(editorAction('alignTopE', undefined, input([a, b]))).toEqual({
			kind: 'intents',
			intents: [],
		});
		expect(editorAction('alignCenterE', undefined, input([a], { pages }))).toMatchObject({
			intents: [{ type: 'MoveFrames', moves: [{ frame: 'a', box: { x: 140 } }] }],
		});
		const spread3 = [at('a', 0), at('b', 10), at('c', 80)];
		expect(editorAction('distH', undefined, input(spread3))).toMatchObject({
			intents: [{ type: 'MoveFrames', moves: [{ frame: 'b', box: { x: 40 } }] }],
		});
	});
});

describe('Book size, Margins and Cover', () => {
	it('opens the size and margins dialogs from the Pages tab', () => {
		const state = editorState(input());
		expect(state.available.has('pageSize')).toBe(true);
		expect(state.available.has('marginsT')).toBe(true);
		expect(editorAction('pageSize', undefined, input())).toEqual({ kind: 'bookSize' });
		expect(editorAction('marginsT', undefined, input())).toEqual({ kind: 'pageMargins' });
	});

	it('goes to the cover only when the book has one', () => {
		expect(editorState(input()).available.has('cover')).toBe(false);
		expect(editorAction('cover', undefined, input())).toBeNull();
		expect(editorState(input([], { hasCover: true })).available.has('cover')).toBe(true);
		expect(editorAction('cover', undefined, input([], { hasCover: true }))).toEqual({
			kind: 'cover',
		});
	});
});

describe('Link text', () => {
	it('is live for one free text box whose words are its own', () => {
		expect(editorState(input([text()])).available.has('thread')).toBe(true);
		expect(editorAction('thread', undefined, input([text()]))).toEqual({
			kind: 'link',
			frame: 't',
		});
		for (const selected of [
			[],
			[text(), text({ id: 'u' })],
			[text({ locked: true })],
			[text({ fromMaster: 'mst_a' })],
			[text({ flow: { id: 'fl', index: 0 } })],
			[image()],
		]) {
			expect(editorState(input(selected)).available.has('thread')).toBe(false);
			expect(editorAction('thread', undefined, input(selected))).toBeNull();
		}
	});
});

describe('the Home tab’s clipboard', () => {
	it('always offers Paste, and Copy and Cut for a selection', () => {
		const none = editorState(input()).available;
		expect(none.has('paste')).toBe(true);
		expect(none.has('pasteSpecial')).toBe(true);
		expect(none.has('copy')).toBe(false);
		expect(none.has('cut')).toBe(false);
		const some = editorState(input([text()])).available;
		expect(some.has('copy') && some.has('cut')).toBe(true);
	});

	it('cannot cut what is locked or a master’s, but can copy it', () => {
		const locked = editorState(input([text({ locked: true })])).available;
		expect(locked.has('copy')).toBe(true);
		expect(locked.has('cut')).toBe(false);
		const fromMaster = editorState(input([text({ fromMaster: 'mst_a' })])).available;
		expect(fromMaster.has('copy')).toBe(true);
		expect(fromMaster.has('cut')).toBe(false);
		expect(editorState(input([text({ locked: true }), image()])).available.has('cut')).toBe(true);
	});

	it('asks for the clipboard, and for a paste in place from the Paste special menu', () => {
		expect(editorAction('copy', undefined, input([text()]))).toEqual({
			kind: 'clipboard',
			op: 'copy',
		});
		expect(editorAction('cut', undefined, input([text()]))).toEqual({
			kind: 'clipboard',
			op: 'cut',
		});
		expect(editorAction('paste', undefined, input())).toEqual({ kind: 'clipboard', op: 'paste' });
		expect(editorState(input()).options.pasteSpecial?.map(([id]) => id)).toEqual([
			'plain',
			'inPlace',
		]);
		expect(editorAction('pasteSpecial', 'inPlace', input())).toEqual({
			kind: 'clipboard',
			op: 'pasteInPlace',
		});
		expect(editorAction('pasteSpecial', 'nonsense', input())).toBeNull();
	});

	it('offers only the words, and in place, with no frame to paste into', () => {
		expect(editorAction('pasteSpecial', 'plain', input())).toEqual({
			kind: 'clipboard',
			op: 'pastePlain',
		});
		expect(editorAction('pasteSpecial', 'into', input())).toBeNull();
		for (const selected of [
			[text(), text({ id: 'u' })],
			[text({ locked: true })],
			[text({ fromMaster: 'mst_a' })],
		]) {
			expect(editorState(input(selected)).options.pasteSpecial?.map(([id]) => id)).not.toContain(
				'into',
			);
			expect(editorAction('pasteSpecial', 'into', input(selected))).toBeNull();
		}
	});

	it('pastes into the one free text or picture frame selected', () => {
		for (const only of [text(), image()]) {
			expect(editorState(input([only])).options.pasteSpecial?.map(([id]) => id)).toEqual([
				'plain',
				'into',
				'inPlace',
			]);
			expect(editorAction('pasteSpecial', 'into', input([only]))).toEqual({
				kind: 'clipboard',
				op: 'pasteInto',
			});
		}
	});
});

describe('Group and Ungroup', () => {
	const shape2 = (id: string) => text({ id });
	const aGroup = (extra = {}) =>
		({
			type: 'group',
			id: 'g',
			box: {},
			locked: false,
			children: [shape2('a'), shape2('b')],
			...extra,
		}) as unknown as Frame;

	it('groups two or more free frames and takes one group apart', () => {
		const two = editorState(input([shape2('a'), shape2('b')])).available;
		expect(two.has('group')).toBe(true);
		expect(two.has('ungroup')).toBe(false);
		expect(editorState(input([shape2('a')])).available.has('group')).toBe(false);
		const one = editorState(input([aGroup()])).available;
		expect(one.has('ungroup')).toBe(true);
		expect(one.has('group')).toBe(false);
	});

	it('leaves them off for locked frames, a master page and a story', () => {
		const has = (selected: Frame[], extra = {}) =>
			editorState(input(selected, extra)).available.has('group');
		expect(has([shape2('a'), text({ id: 'b', locked: true })])).toBe(false);
		expect(has([shape2('a'), shape2('b')], { master: true })).toBe(false);
		expect(has([shape2('a'), text({ id: 'b', flow: { id: 'fl', index: 0 } })])).toBe(false);
		expect(editorState(input([aGroup({ locked: true })])).available.has('ungroup')).toBe(false);
	});

	it('asks for the group or its undoing', () => {
		const frames = [shape2('a'), shape2('b')];
		expect(editorAction('group', undefined, input(frames))).toEqual({ kind: 'group', frames });
		expect(editorAction('group', undefined, input([shape2('a')]))).toBeNull();
		expect(editorAction('ungroup', undefined, input([aGroup()]))).toMatchObject({
			kind: 'ungroup',
			frame: { id: 'g' },
		});
		expect(editorAction('ungroup', undefined, input([shape2('a')]))).toBeNull();
	});
});

describe('a look set with the caret alone in the words', () => {
	const frame = text({
		content: {
			inline: [
				{ text: 'plain ', style: null, bold: false, italic: false },
				{ text: 'bold', style: null, bold: true, italic: false },
			],
		},
	});

	it('is for the words typed next, and toggles from the look at the caret', () => {
		const at = (offset: number, pending = null) =>
			input([frame], { caret: { frame: 't', offset }, pending });
		expect(editorAction('bold', undefined, at(3))).toEqual({
			kind: 'pending',
			frame: 't',
			change: { bold: true },
		});
		expect(editorAction('bold', undefined, at(9))).toEqual({
			kind: 'pending',
			frame: 't',
			change: { bold: false },
		});
		// Pressed twice it goes back.
		expect(editorAction('bold', undefined, at(3, { bold: true } as never))).toEqual({
			kind: 'pending',
			frame: 't',
			change: { bold: false },
		});
		expect(editorAction('colour', '#a23b2a', at(3))).toEqual({
			kind: 'pending',
			frame: 't',
			change: { colour: '#a23b2a' },
		});
	});

	it('is not used when words are selected, which are styled instead', () => {
		const action = editorAction(
			'bold',
			undefined,
			input([frame], {
				caret: { frame: 't', offset: 3 },
				selection: { frame: 't', start: 0, end: 3 },
			}),
		);
		expect(action).toMatchObject({ kind: 'intents' });
	});
});
