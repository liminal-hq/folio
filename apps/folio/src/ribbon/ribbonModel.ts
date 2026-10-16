// Builds the ribbon and the classic toolbar from the layout, the settings and what can be run now
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type {
	RibbonItem,
	RibbonModel,
	RibbonTab,
	ToolbarModel,
} from '@liminal-hq/folio-ribbon/types';
import type { Settings } from '@liminal-hq/folio-plugin-settings';
import type { CommandId } from '../commands/appMenuModel';
import { t, type MessageKey } from '../i18n/messages';
import type { EditorState } from './editorCommands';
import { RIBBON_ICONS } from './ribbonIcons';
import { RIBBON_LAYOUT, type LayoutItem } from './ribbonLayout';

/** The menu command each ribbon control runs. A control missing here is listed but unavailable. */
export const RIBBON_COMMANDS: Readonly<Record<string, CommandId>> = {
	newBook: 'file.new',
	open: 'file.open',
	save: 'file.save',
	undo: 'edit.undo',
	redo: 'edit.redo',
	export: 'file.export',
	exportBook: 'file.export',
	zoomIn: 'view.zoomIn',
	zoomOut: 'view.zoomOut',
	menubar: 'view.menuBar',
	ribbonToggle: 'view.ribbon',
};

/** What a control shows, apart from its glyph: the settings it reflects and what can be run. */
export interface RibbonContext {
	settings: Settings;
	available: ReadonlySet<CommandId>;
	/** The document's controls, when a book is open: which are live and how they are set. */
	editor?: EditorState;
}

/** The glyphs of classic toolbar controls that are not named after their id. */
const CLASSIC_ICONS: Readonly<Record<string, string>> = {
	kindle: 'ereader',
	kobo: 'ereader',
	ribbonToggle: 'ribbon',
	exportBook: 'export',
	style: 'tabs',
};

const label = (id: string) => t(`ribbon.item.${id}` as MessageKey);

/** The File button's menu: the commands the backstage will gather. */
const FILE_COMMANDS: readonly [CommandId, MessageKey][] = [
	['file.new', 'ribbon.file.new'],
	['file.open', 'ribbon.file.open'],
	['file.save', 'ribbon.file.save'],
	['file.saveAs', 'ribbon.file.saveAs'],
	['file.export', 'ribbon.file.export'],
	['file.close', 'ribbon.file.close'],
];

/** The File menu's icons: the ribbon's own paths where it has them, and two more for Save as and Close. */
const FILE_ICONS: Readonly<Record<string, string>> = {
	'file.new': RIBBON_ICONS.newBook!,
	'file.open': RIBBON_ICONS.open!,
	'file.save': RIBBON_ICONS.save!,
	'file.saveAs': 'M3 2h8l3 3v3M3 2a1 1 0 00-1 1v10a1 1 0 001 1h4M5 2v4h5V2M10 14l4-4 1 1-4 4h-1z',
	'file.export': RIBBON_ICONS.export!,
	'file.close': 'M3 3l10 10M13 3L3 13',
};

function isAvailable(id: string, context: RibbonContext): boolean {
	const command = RIBBON_COMMANDS[id];
	if (command !== undefined && context.available.has(command)) return true;
	return context.editor?.available.has(id) ?? false;
}

function toRibbonItem(
	source: LayoutItem,
	context: RibbonContext,
	icons: Readonly<Record<string, string>> = {},
): RibbonItem {
	const enabled = isAvailable(source.id, context);
	const kind = source.kind ?? 'button';
	const item: RibbonItem = {
		id: source.id,
		label: label(source.id),
		d: RIBBON_ICONS[icons[source.id] ?? source.icon ?? source.id] ?? '',
		size: source.large ? 'large' : 'small',
		kind,
		disabled: !enabled,
		...(enabled ? {} : { desc: t('ribbon.unavailable') }),
		...(source.expert ? { expert: true } : {}),
		...(source.radio ? { radio: source.radio } : {}),
	};
	if (source.id === 'menubar') item.active = context.settings.interface.menuBar;
	if (source.id === 'spell') item.active = context.settings.spelling.enabled;
	const pressed = context.editor?.active[source.id];
	if (kind === 'toggle' && pressed !== undefined) item.active = pressed;
	const shown = context.editor?.values[source.id];
	if (shown !== undefined) item.value = shown;
	if (kind === 'select') {
		item.options = [['', t(`ribbon.value.${source.id}` as MessageKey)]];
		item.width = source.id === 'font' ? 150 : 130;
	}
	if (source.id === 'size') Object.assign(item, { min: 4, max: 96, step: 1, defaultValue: 12 });
	return item;
}

/** The ribbon: File, then the tabs of spec 06. Contextual tabs show when the selection calls for them. */
export function buildRibbonModel(context: RibbonContext): RibbonModel {
	const map = (item: LayoutItem) => toRibbonItem(item, context);
	const tabs: RibbonTab[] = RIBBON_LAYOUT.map((tab) => ({
		id: tab.id,
		label: t(`ribbon.tab.${tab.id}` as MessageKey),
		...(tab.contextual ? { contextual: true } : {}),
		groups: tab.groups.map((group) => ({
			id: group.id,
			label: t(`ribbon.group.${group.id}` as MessageKey),
			...(group.expert ? { expert: true } : {}),
			...(group.items ? { items: group.items.map(map) } : {}),
			...(group.rows
				? { rows: group.rows.map((row) => row.map((cluster) => cluster.map(map))) }
				: {}),
		})),
	}));
	return {
		file: {
			label: t('ribbon.file'),
			items: FILE_COMMANDS.filter(([id]) => context.available.has(id)).map(
				([id, key]): [string, string, string?] => [id, t(key), FILE_ICONS[id]],
			),
		},
		tabs,
	};
}

/** The classic toolbar's groups (spec 06). */
const CLASSIC_GROUPS: readonly (readonly LayoutItem[])[] = [
	[{ id: 'newBook' }, { id: 'open' }, { id: 'save' }],
	[{ id: 'undo' }, { id: 'redo' }],
	[{ id: 'text' }, { id: 'image' }, { id: 'shape' }],
	[{ id: 'style', kind: 'menu' }],
	[
		{ id: 'bold', kind: 'toggle' },
		{ id: 'italic', kind: 'toggle' },
		{ id: 'alignL', kind: 'toggle', radio: 'align' },
		{ id: 'alignC', kind: 'toggle', radio: 'align' },
		{ id: 'alignR', kind: 'toggle', radio: 'align' },
	],
	[{ id: 'front' }, { id: 'back' }],
	[
		{ id: 'margins', kind: 'toggle' },
		{ id: 'grid', kind: 'toggle' },
		{ id: 'bleed', kind: 'toggle' },
		{ id: 'snap', kind: 'toggle' },
	],
	[
		{ id: 'print', kind: 'toggle', radio: 'preview' },
		{ id: 'kindle', kind: 'toggle', radio: 'preview' },
		{ id: 'kobo', kind: 'toggle', radio: 'preview' },
	],
	[{ id: 'zoomOut' }, { id: 'fit' }, { id: 'zoomIn' }],
	[{ id: 'ribbonToggle' }],
];

export function buildToolbarModel(context: RibbonContext): ToolbarModel {
	return {
		groups: CLASSIC_GROUPS.map((group) =>
			group.map((item) => toRibbonItem(item, context, CLASSIC_ICONS)),
		),
	};
}

export { label as ribbonItemLabel };
