// The app menu in the title bar's start slot: the primary menu while the menu bar is hidden
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { AppMenuButton } from '@liminal-hq/folio-chrome/TitleBar/AppMenuButton';
import type { MenuItem, SelectableMenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import { APP_MENU_MNEMONICS } from '../commands/appMenuModel';
import { t } from '../i18n/messages';
import { AppMarkIcon } from '../icons/AppIcons';

interface AppMenuProps {
	items: MenuItem[];
	onSelect: (item: SelectableMenuItem) => void;
	/**
	 * The menu bar is showing and carries the menus: the mark and name stay as plain text that
	 * opens nothing, and the keys are the bar's.
	 */
	menuBar?: boolean;
}

/** F10 or a lone Alt toggles it, and Alt+F, E, V and H open it with the menu named by the letter open. */
export function AppMenu({ items, onSelect, menuBar = false }: AppMenuProps) {
	return (
		<AppMenuButton
			label={t('app.name')}
			mark={<AppMarkIcon />}
			items={items}
			onSelect={onSelect}
			mnemonics={APP_MENU_MNEMONICS}
			interactive={!menuBar}
		/>
	);
}
