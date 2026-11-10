// Tests for the ribbon and classic toolbar models
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RibbonItem } from '@liminal-hq/folio-ribbon/types';
import { describe, expect, it } from 'vitest';
import type { CommandId } from '../commands/appMenuModel';
import { messages } from '../i18n/messages';
import type { Settings } from '@liminal-hq/folio-protocol/generated/Settings';
import { DEFAULT_SETTINGS } from '../settings/fakeSettingsClient';
import { DEFAULT_MARKS } from '../canvas/marksModel';
import { TOOL_COMMANDS } from '../commands/toolCommands';
import { TOOLS } from '../booktools/registry';
import { editorState } from './editorCommands';
import { RIBBON_ICONS } from './ribbonIcons';
import { RIBBON_LAYOUT } from './ribbonLayout';
import { RIBBON_COMMANDS, buildRibbonModel, buildToolbarModel } from './ribbonModel';

const none = new Set<CommandId>();
const some = new Set<CommandId>(['edit.undo', 'file.save', 'view.menuBar']);

function allItems(items: ReturnType<typeof buildRibbonModel>): RibbonItem[] {
	return items.tabs.flatMap((tab) =>
		tab.groups.flatMap((group) => [...(group.items ?? []), ...(group.rows ?? []).flat(2)]),
	);
}

describe('the ribbon layout', () => {
	it('has the tabs of spec 06 and the three contextual tabs', () => {
		expect(RIBBON_LAYOUT.map((tab) => tab.id)).toEqual([
			'home',
			'insert',
			'pages',
			'layout',
			'view',
			'tools',
			'export',
			'picture',
			'shape',
			'table',
		]);
		expect(RIBBON_LAYOUT.filter((tab) => tab.contextual).map((tab) => tab.id)).toEqual([
			'picture',
			'shape',
			'table',
		]);
	});

	it('has a message for every tab, group and control, and a glyph for every control', () => {
		for (const tab of RIBBON_LAYOUT) {
			expect(messages).toHaveProperty([`ribbon.tab.${tab.id}`]);
			for (const group of tab.groups) {
				expect(messages).toHaveProperty([`ribbon.group.${group.id}`]);
				const items = [...(group.items ?? []), ...(group.rows ?? []).flat(2)];
				for (const item of items) {
					expect(messages, item.id).toHaveProperty([`ribbon.item.${item.id}`]);
					expect(RIBBON_ICONS, item.id).toHaveProperty([item.icon ?? item.id]);
				}
			}
		}
	});

	it('marks the expert groups and items', () => {
		const model = buildRibbonModel({ settings: DEFAULT_SETTINGS, available: none });
		const pages = model.tabs.find((tab) => tab.id === 'pages');
		expect(pages?.groups.find((g) => g.id === 'pages.masters')?.expert).toBe(true);
		expect(allItems(model).find((item) => item.id === 'justify')?.expert).toBe(true);
		// It shows pressed while a master is open, which only a toggle can.
		expect(allItems(model).find((item) => item.id === 'master')?.kind).toBe('toggle');
	});
});

describe('buildRibbonModel', () => {
	it('lists a control as unavailable unless it runs a command that can run', () => {
		const model = buildRibbonModel({ settings: DEFAULT_SETTINGS, available: some });
		const items = allItems(model);
		const undo = items.find((i) => i.id === 'undo');
		const redo = items.find((i) => i.id === 'redo');
		const bold = items.find((i) => i.id === 'bold');
		expect(undo?.disabled).toBe(false);
		expect(redo?.disabled).toBe(true);
		expect(redo?.desc).toBe('Not available yet');
		expect(bold?.disabled).toBe(true);
	});

	it('reflects the menu bar setting on its toggle', () => {
		const on = {
			...DEFAULT_SETTINGS,
			interface: { ...DEFAULT_SETTINGS.interface, menuBar: true },
		};
		const find = (settings: typeof on) =>
			allItems(buildRibbonModel({ settings, available: some })).find((i) => i.id === 'menubar');
		expect(find(on)?.active).toBe(true);
		expect(find(DEFAULT_SETTINGS)?.active).toBe(false);
	});

	it('reflects the spelling setting on its toggle', () => {
		const off = { ...DEFAULT_SETTINGS, spelling: { enabled: false, personal: [] } };
		const find = (settings: Settings) =>
			allItems(buildRibbonModel({ settings, available: some })).find((i) => i.id === 'spell');
		expect(find(DEFAULT_SETTINGS)?.kind).toBe('toggle');
		expect(find(DEFAULT_SETTINGS)?.active).toBe(true);
		expect(find(off)?.active).toBe(false);
	});

	it('puts the commands that can run in the File menu', () => {
		const model = buildRibbonModel({ settings: DEFAULT_SETTINGS, available: some });
		expect(model.file?.items?.map(([id, label]) => [id, label])).toEqual([['file.save', 'Save']]);
	});

	it('gives every File menu command an icon', () => {
		const every = new Set<CommandId>([
			'file.new',
			'file.open',
			'file.save',
			'file.saveAs',
			'file.export',
			'file.close',
		]);
		const items = buildRibbonModel({ settings: DEFAULT_SETTINGS, available: every }).file?.items;
		expect(items).toHaveLength(6);
		for (const [id, , d] of items ?? []) expect(d, id).toMatch(/^M/);
	});

	it('leaves values to the ribbon: no invented size and no placeholder value on unavailable selects', () => {
		const items = allItems(buildRibbonModel({ settings: DEFAULT_SETTINGS, available: none }));
		const size = items.find((i) => i.id === 'size');
		expect(size?.value).toBeUndefined();
		expect(size?.disabled).toBe(true);
		const selects = items.filter((i) => i.kind === 'select');
		expect(selects.length).toBeGreaterThan(0);
		for (const select of selects) {
			expect(select.value, select.id).toBeUndefined();
			expect(select.disabled, select.id).toBe(true);
		}
	});

	it('gives each control a glyph, a label and a size', () => {
		const items = allItems(buildRibbonModel({ settings: DEFAULT_SETTINGS, available: none }));
		for (const item of items) {
			expect(item.d, item.id).not.toBe('');
			expect(item.label, item.id).not.toBe('');
		}
		expect(items.find((i) => i.id === 'text')?.size).toBe('large');
	});
});

describe('the menus of the ribbon', () => {
	it('gives the menus that are live their choices, so none opens empty', () => {
		const editor = editorState({
			spread: { id: 'sp', kind: 'spread', pages: [], frames: [], guides: [] } as never,
			spreadIndex: 1,
			spreadCount: 3,
			selected: [],
			layout: null,
			choices: [],
			marks: DEFAULT_MARKS,
			snap: true,
			rulers: false,
			jax: true,
		});
		const items = allItems(
			buildRibbonModel({ settings: DEFAULT_SETTINGS, available: none, editor }),
		);
		for (const id of ['rulerUnits', 'toolbarLayout', 'shapeMenu']) {
			const item = items.find((i) => i.id === id);
			expect(item?.kind, id).toBe('menu');
			expect(item?.disabled, id).toBe(false);
			expect(item?.options?.length, id).toBeGreaterThan(1);
		}
		expect(items.find((i) => i.id === 'rulerUnits')?.options).toContainEqual([
			'millimetres',
			'Millimetres',
		]);
		for (const id of ['baseline', 'columns', 'shape', 'askJax']) {
			expect(items.find((i) => i.id === id)?.disabled, id).toBe(false);
		}
	});

	it('opens Contents in Book tools, and leaves the unbuilt layouts unavailable', () => {
		expect(RIBBON_COMMANDS.toc).toBe('tools.structure');
		const items = allItems(
			buildRibbonModel({
				settings: DEFAULT_SETTINGS,
				available: new Set<CommandId>(['tools.structure']),
			}),
		);
		expect(items.find((i) => i.id === 'toc')?.disabled).toBe(false);
		expect(items.find((i) => i.id === 'sidebarLayout')?.disabled).toBe(true);
		expect(items.find((i) => i.id === 'importLayout')?.disabled).toBe(true);
	});
});

describe('the Home tab', () => {
	it('starts with the clipboard: a large Paste, Paste special below it, then Copy and Cut', () => {
		const home = RIBBON_LAYOUT.find((tab) => tab.id === 'home')!;
		expect(home.groups[0]!.id).toBe('home.clipboard');
		expect(home.groups[0]!.items!.map((i) => [i.id, i.kind ?? 'button', !!i.large])).toEqual([
			['paste', 'button', true],
			['pasteSpecial', 'menu', false],
			['copy', 'button', false],
			['cut', 'button', false],
		]);
	});
});

describe('the Font list', () => {
	it('carries the heading of each font through to the select', () => {
		const editor = editorState({
			spread: { id: 'sp', kind: 'spread', pages: [], frames: [], guides: [] } as never,
			spreadIndex: 1,
			spreadCount: 3,
			selected: [],
			layout: null,
			choices: [],
			marks: DEFAULT_MARKS,
			snap: true,
			rulers: false,
			fonts: [
				{ family: 'Literata', origin: 'bundled' },
				{ family: 'Arial', origin: 'system' },
			],
		});
		const font = allItems(
			buildRibbonModel({ settings: DEFAULT_SETTINGS, available: none, editor }),
		).find((i) => i.id === 'font');
		expect(font?.options).toEqual([
			['', 'Style’s font'],
			['Literata', 'Literata', 'In this book'],
			['Arial', 'Arial', 'Installed'],
		]);
	});
});

describe('the Tools tab', () => {
	const tools = RIBBON_LAYOUT.find((tab) => tab.id === 'tools')!;
	const items = tools.groups.flatMap((g) => g.items ?? []);

	it('sits between View and Export with a group for each area of tools', () => {
		const ids = RIBBON_LAYOUT.map((tab) => tab.id);
		expect(ids.indexOf('tools')).toBe(ids.indexOf('view') + 1);
		expect(ids.indexOf('export')).toBe(ids.indexOf('tools') + 1);
		expect(tools.groups.map((g) => g.id)).toEqual([
			'tools.structure',
			'tools.typesetting',
			'tools.pictures',
			'tools.publishing',
			'tools.proofing',
		]);
	});

	it('opens each of the nine tools through its own command', () => {
		const opening = items.filter((i) => i.id !== 'toolSpelling' && i.id !== 'toolFind');
		expect(opening).toHaveLength(9);
		for (const item of opening) {
			const command = RIBBON_COMMANDS[item.id];
			expect(command, item.id).toMatch(/^tool\./);
			expect(command && TOOL_COMMANDS[command as keyof typeof TOOL_COMMANDS], item.id).toBeTruthy();
		}
		// Every tool that is built has a button, once.
		expect(new Set(opening.map((i) => RIBBON_COMMANDS[i.id])).size).toBe(9);
		expect(new Set(Object.values(TOOL_COMMANDS))).toEqual(new Set(TOOLS.map((tool) => tool.id)));
	});

	it('keeps its ids apart from the Home tab, whose Spelling and Find stay', () => {
		const home = RIBBON_LAYOUT.find((tab) => tab.id === 'home')!;
		const homeIds = home.groups
			.flatMap((g) => [...(g.items ?? []), ...(g.rows ?? []).flat(2)])
			.map((i) => i.id);
		expect(homeIds).toContain('spell');
		expect(homeIds).toContain('find');
		for (const item of items) expect(homeIds).not.toContain(item.id);
	});

	it('runs the tools without a selection, switches Spelling and leaves Find unavailable', () => {
		const available = new Set<CommandId>(Object.keys(TOOL_COMMANDS) as CommandId[]);
		const editor = editorState({
			spread: { id: 'sp', kind: 'spread', pages: [], frames: [], guides: [] } as never,
			spreadIndex: 1,
			spreadCount: 3,
			selected: [],
			layout: null,
			choices: [],
			marks: DEFAULT_MARKS,
			snap: true,
			rulers: false,
		});
		const shown = allItems(buildRibbonModel({ settings: DEFAULT_SETTINGS, available, editor }));
		const byId = (id: string) => shown.find((i) => i.id === id)!;
		for (const item of items.filter((i) => i.id !== 'toolSpelling' && i.id !== 'toolFind')) {
			expect(byId(item.id).disabled, item.id).toBe(false);
		}
		expect(byId('toolSpelling').disabled).toBe(false);
		expect(byId('toolSpelling').active).toBe(true);
		expect(byId('toolFind').disabled).toBe(true);
		expect(byId('toolFind').desc).toBe('Not available yet');
	});

	it('marks the technical tool for Expert mode', () => {
		expect(items.filter((i) => i.expert).map((i) => i.id)).toEqual(['toolStyles']);
	});
});

describe('buildToolbarModel', () => {
	it('has the groups of the classic toolbar', () => {
		const model = buildToolbarModel({ settings: DEFAULT_SETTINGS, available: none });
		expect(model.groups.map((g) => g.map((i) => i.id))).toEqual([
			['newBook', 'open', 'save'],
			['undo', 'redo'],
			['text', 'image', 'shape'],
			['style'],
			['bold', 'italic', 'alignL', 'alignC', 'alignR'],
			['front', 'back'],
			['margins', 'grid', 'bleed', 'snap'],
			['print', 'kindle', 'kobo'],
			['zoomOut', 'fit', 'zoomIn'],
			['ribbonToggle'],
		]);
	});

	it('runs the ribbon toggle through the ribbon command', () => {
		const model = buildToolbarModel({
			settings: DEFAULT_SETTINGS,
			available: new Set<CommandId>(['view.ribbon']),
		});
		expect(RIBBON_COMMANDS.ribbonToggle).toBe('view.ribbon');
		expect(model.groups.flat().find((i) => i.id === 'ribbonToggle')?.disabled).toBe(false);
	});

	it('makes the ribbon toggle a plain button, so it keeps no pressed state of its own', () => {
		const model = buildToolbarModel({ settings: DEFAULT_SETTINGS, available: none });
		const item = model.groups.flat().find((i) => i.id === 'ribbonToggle');
		expect(item?.kind).toBe('button');
		expect(item?.active).toBeUndefined();
	});
});
