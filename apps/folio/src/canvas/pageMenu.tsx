// The menu for the bare page: add things to it, and switch the page helpers
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import { AddPagesIcon, DuplicateIcon } from '../icons/AppIcons';
import { t } from '../i18n/messages';
import type { Marks } from './SpreadCanvas';

export type PageCommand =
	| 'addText'
	| 'addPicture'
	| 'addNote'
	| 'addPages'
	| 'duplicateSpread'
	| 'toggleMargins'
	| 'toggleGrid';

const Box = () => (
	<svg
		width="16"
		height="16"
		viewBox="0 0 16 16"
		fill="none"
		stroke="currentColor"
		strokeWidth="1.3"
		aria-hidden="true"
	>
		<rect x="2.5" y="3.5" width="11" height="9" rx="1.2" />
	</svg>
);

export function pageMenuItems(marks: Marks, canDuplicate: boolean): MenuItem[] {
	return [
		{ type: 'action', id: 'addText', label: t('page.addText'), icon: <Box /> },
		{ type: 'action', id: 'addPicture', label: t('page.addPicture'), icon: <Box /> },
		{ type: 'action', id: 'addNote', label: t('page.addNote'), icon: <Box /> },
		{ type: 'separator' },
		{ type: 'action', id: 'addPages', label: t('page.addPages'), icon: <AddPagesIcon /> },
		{
			type: 'action',
			id: 'duplicateSpread',
			label: t('page.duplicateSpread'),
			icon: <DuplicateIcon />,
			disabled: !canDuplicate,
		},
		{ type: 'separator' },
		{ type: 'checkbox', id: 'toggleMargins', label: t('page.showMargins'), checked: marks.margins },
		{
			type: 'checkbox',
			id: 'toggleGrid',
			label: t('page.showGrid'),
			checked: marks.columns || marks.baseline,
		},
	];
}
