// The app menu's rows and what choosing one does, for the window's settings
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem, SelectableMenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import { useWindowControls } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { useCallback, useMemo } from 'react';
import { buildAppMenu, buildMenuBar } from '../commands/appMenuModel';
import { ENABLED_COMMANDS, runCommand } from '../commands/runCommand';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import { useWindows } from './WindowsContext';

const AVAILABLE = { enabled: ENABLED_COMMANDS };

export interface AppMenuModel {
	/** The app menu's rows. */
	items: MenuItem[];
	/** The menu bar's menus. */
	barItems: MenuItem[];
	onSelect: (item: SelectableMenuItem) => void;
}

export function useAppMenu(): AppMenuModel {
	const settings = useSettings();
	const update = useUpdateSettings();
	const controls = useWindowControls();
	const windows = useWindows();

	const items = useMemo(() => buildAppMenu(settings, AVAILABLE), [settings]);
	const barItems = useMemo(() => buildMenuBar(settings, AVAILABLE), [settings]);
	const onSelect = useCallback(
		(item: SelectableMenuItem) => {
			void runCommand(item.id, {
				settings,
				update,
				openWindow: (kind) => windows.open(kind),
				closeWindow: () => controls.close(),
			});
		},
		[settings, update, controls, windows],
	);

	return { items, barItems, onSelect };
}
