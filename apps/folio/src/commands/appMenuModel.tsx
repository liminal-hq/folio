// The app menu and the menu bar's menus as data, built from the settings in force
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import type { Settings } from '@liminal-hq/folio-plugin-settings';
import type { ReactNode } from 'react';
import { t } from '../i18n/messages';
import {
	AboutIcon,
	BookToolsIcon,
	CloseBookIcon,
	EditIcon,
	ExpertIcon,
	ExportIcon,
	ImportIcon,
	JaxIcon,
	KeyboardIcon,
	MenuBarIcon,
	NewBookIcon,
	OpenIcon,
	PreferencesIcon,
	QuitIcon,
	RecentIcon,
	RedoIcon,
	RibbonIcon,
	SaveIcon,
	UndoIcon,
	ViewIcon,
	ZoomInIcon,
	ZoomOutIcon,
} from '../icons/AppIcons';

/** Every command the menus can run. A command with no behaviour yet is listed as unavailable. */
export type CommandId =
	| 'file.new'
	| 'file.open'
	| 'file.openRecent'
	| 'file.importText'
	| 'file.importLayout'
	| 'file.save'
	| 'file.close'
	| 'file.saveAs'
	| 'file.export'
	| 'edit.undo'
	| 'edit.redo'
	| 'view.menuBar'
	| 'view.ribbon'
	| 'view.expert'
	| 'view.jax'
	| 'view.zoomIn'
	| 'view.zoomOut'
	| 'tools.structure'
	| 'tools.typesetting'
	| 'tools.pictures'
	| 'tools.publishing'
	| 'app.preferences'
	| 'app.shortcuts'
	| 'app.agents'
	| 'app.updates'
	| 'app.about'
	| 'app.quit';

/** The id prefix of a recent book's row; the rest is its path. */
export const RECENT_PREFIX = 'file.openRecent:';

/** Alt plus a letter opens the app menu with that menu open. */
export const APP_MENU_MNEMONICS = {
	f: 'menu:file',
	e: 'menu:edit',
	v: 'menu:view',
	t: 'menu:tools',
	h: 'menu:help',
};

/** The commands that do something today; the others are listed but unavailable. */
export interface CommandAvailability {
	/** Commands with behaviour wired up. */
	enabled: ReadonlySet<CommandId>;
}

function action(
	id: CommandId,
	label: string,
	icon: ReactNode,
	available: CommandAvailability,
	shortcut?: string,
): MenuItem {
	const enabled = available.enabled.has(id);
	return {
		type: 'action',
		id,
		label,
		icon,
		...(shortcut ? { shortcut } : {}),
		disabled: !enabled,
		...(enabled ? {} : { title: t('menu.unavailable') }),
	};
}

function toggle(
	id: CommandId,
	label: string,
	icon: ReactNode,
	checked: boolean,
	available: CommandAvailability,
): MenuItem {
	const enabled = available.enabled.has(id);
	return {
		type: 'checkbox',
		id,
		label,
		icon,
		checked,
		disabled: !enabled,
		...(enabled ? {} : { title: t('menu.unavailable') }),
	};
}

function recentBooks(settings: Settings): MenuItem[] {
	if (settings.recentBooks.length === 0) {
		return [
			{
				type: 'action',
				id: 'file.openRecent.none',
				label: t('command.noRecent'),
				icon: <RecentIcon />,
				disabled: true,
			},
		];
	}
	return settings.recentBooks.map((path) => ({
		type: 'action' as const,
		id: `${RECENT_PREFIX}${path}`,
		label:
			path
				.split(/[/\\]/)
				.pop()
				?.replace(/\.folio$/i, '') ?? path,
		icon: <RecentIcon />,
		title: path,
	}));
}

/** The File menu's rows, shared by the app menu and the menu bar. */
function fileItems(settings: Settings, available: CommandAvailability): MenuItem[] {
	return [
		action('file.new', t('command.new'), <NewBookIcon />, available, 'Ctrl+N'),
		action('file.open', t('command.open'), <OpenIcon />, available, 'Ctrl+O'),
		{
			type: 'submenu',
			id: 'file.openRecent',
			label: t('command.openRecent'),
			icon: <RecentIcon />,
			items: recentBooks(settings),
		},
		{ type: 'separator' },
		action('file.importText', t('command.importText'), <ImportIcon />, available),
		action('file.importLayout', t('command.importLayout'), <ImportIcon />, available),
		{ type: 'separator' },
		action('file.save', t('command.save'), <SaveIcon />, available, 'Ctrl+S'),
		action('file.saveAs', t('command.saveAs'), <SaveIcon />, available, 'Ctrl+Shift+S'),
		action('file.export', t('command.export'), <ExportIcon />, available, 'Ctrl+E'),
		{ type: 'separator' },
		action('file.close', t('command.close'), <CloseBookIcon />, available, 'Ctrl+W'),
	];
}

function interfaceToggles(settings: Settings, available: CommandAvailability): MenuItem[] {
	return [
		toggle(
			'view.menuBar',
			t('command.showMenuBar'),
			<MenuBarIcon />,
			settings.interface.menuBar,
			available,
		),
		toggle(
			'view.ribbon',
			t('command.useRibbon'),
			<RibbonIcon />,
			settings.interface.toolbar === 'ribbon',
			available,
		),
		toggle(
			'view.expert',
			t('command.expertMode'),
			<ExpertIcon />,
			settings.mode === 'expert',
			available,
		),
		toggle('view.jax', t('command.jax'), <JaxIcon />, settings.jax !== 'off', available),
	];
}

/** The groups of Book tools, each opening the tools window on it. */
function toolItems(available: CommandAvailability): MenuItem[] {
	return [
		action('tools.structure', t('command.tools.structure'), <BookToolsIcon />, available),
		action('tools.typesetting', t('command.tools.typesetting'), <BookToolsIcon />, available),
		action('tools.pictures', t('command.tools.pictures'), <BookToolsIcon />, available),
		action('tools.publishing', t('command.tools.publishing'), <BookToolsIcon />, available),
	];
}

function helpItems(available: CommandAvailability): MenuItem[] {
	return [
		action('app.shortcuts', t('command.shortcuts'), <KeyboardIcon />, available, 'Ctrl+?'),
		action('app.agents', t('command.agents'), <BookToolsIcon />, available),
		action('app.updates', t('command.updates'), <AboutIcon />, available),
		action('app.about', t('command.about'), <AboutIcon />, available),
	];
}

/** The app menu: the primary menu while the menu bar is hidden (spec 06). */
export function buildAppMenu(settings: Settings, available: CommandAvailability): MenuItem[] {
	return [
		...fileItems(settings, available),
		{ type: 'separator' },
		...interfaceToggles(settings, available),
		{ type: 'separator' },
		{
			type: 'submenu',
			id: 'menu:book-tools',
			label: t('command.bookTools'),
			icon: <BookToolsIcon />,
			items: toolItems(available),
		},
		action('app.preferences', t('command.preferences'), <PreferencesIcon />, available, 'Ctrl+,'),
		...helpItems(available),
		{ type: 'separator' },
		action('app.quit', t('command.quit'), <QuitIcon />, available, 'Ctrl+Q'),
	];
}

/** The menu bar's menus: File, Edit, View, Tools and Help. */
export function buildMenuBar(settings: Settings, available: CommandAvailability): MenuItem[] {
	return [
		{
			type: 'submenu',
			id: 'menu:file',
			label: t('menu.file'),
			icon: <OpenIcon />,
			items: [
				...fileItems(settings, available),
				{ type: 'separator' },
				action('app.quit', t('command.quit'), <QuitIcon />, available, 'Ctrl+Q'),
			],
		},
		{
			type: 'submenu',
			id: 'menu:edit',
			label: t('menu.edit'),
			icon: <EditIcon />,
			items: [
				action('edit.undo', t('command.undo'), <UndoIcon />, available, 'Ctrl+Z'),
				action('edit.redo', t('command.redo'), <RedoIcon />, available, 'Ctrl+Shift+Z'),
				{ type: 'separator' },
				action(
					'app.preferences',
					t('command.preferences'),
					<PreferencesIcon />,
					available,
					'Ctrl+,',
				),
			],
		},
		{
			type: 'submenu',
			id: 'menu:view',
			label: t('menu.view'),
			icon: <ViewIcon />,
			items: [
				...interfaceToggles(settings, available),
				{ type: 'separator' },
				action('view.zoomIn', t('command.zoomIn'), <ZoomInIcon />, available, 'Ctrl+='),
				action('view.zoomOut', t('command.zoomOut'), <ZoomOutIcon />, available, 'Ctrl+-'),
			],
		},
		{
			type: 'submenu',
			id: 'menu:tools',
			label: t('menu.tools'),
			icon: <BookToolsIcon />,
			items: toolItems(available),
		},
		{
			type: 'submenu',
			id: 'menu:help',
			label: t('menu.help'),
			icon: <AboutIcon />,
			items: helpItems(available),
		},
	];
}
