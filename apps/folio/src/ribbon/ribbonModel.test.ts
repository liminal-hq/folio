// Tests for the ribbon and classic toolbar models
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RibbonItem } from '@liminal-hq/folio-ribbon/types';
import { describe, expect, it } from 'vitest';
import type { CommandId } from '../commands/appMenuModel';
import { messages } from '../i18n/messages';
import { DEFAULT_SETTINGS } from '../settings/fakeSettingsClient';
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

	it('puts the commands that can run in the File menu', () => {
		const model = buildRibbonModel({ settings: DEFAULT_SETTINGS, available: some });
		expect(model.file?.items).toEqual([['file.save', 'Save']]);
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
