// The menu for the bare page: add things to it, and switch the page helpers
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import { AddPagesIcon, DuplicateIcon } from '../icons/AppIcons';
import { t } from '../i18n/messages';
import type { MasterItem } from './masterItems';
import type { Marks } from './marksModel';

export type PageCommand =
	| 'addText'
	| 'addPicture'
	| 'addNote'
	| 'addPages'
	| 'duplicateSpread'
	| 'toggleMargins'
	| 'toggleGrid'
	/** `editMaster:<master>` opens that master page to edit. */
	| `editMaster:${string}`
	/** `hideMaster:<page>:<frame>` hides a master item on one page; `showMaster:…` shows it again. */
	| `${'hideMaster' | 'showMaster'}:${string}:${string}`
	/** `copyMaster:<page>:<frame>` hides a master item on one page and leaves an editable copy there. */
	| `copyMaster:${string}:${string}`;

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

export function pageMenuItems(
	marks: Marks,
	canDuplicate: boolean,
	masterItems: MasterItem[] = [],
	/** On a master page there are no more pages to add, and no master items to hide. */
	onMaster = false,
	editMaster: { id: string; name: string } | null = null,
): MenuItem[] {
	const pages: MenuItem[] = onMaster
		? []
		: [
				{ type: 'separator' },
				{ type: 'action', id: 'addPages', label: t('page.addPages'), icon: <AddPagesIcon /> },
				{
					type: 'action',
					id: 'duplicateSpread',
					label: t('page.duplicateSpread'),
					icon: <DuplicateIcon />,
					disabled: !canDuplicate,
				},
			];
	const hide = onMaster ? [] : masterItems;
	return [
		{ type: 'action', id: 'addText', label: t('page.addText'), icon: <Box /> },
		{ type: 'action', id: 'addPicture', label: t('page.addPicture'), icon: <Box /> },
		{ type: 'action', id: 'addNote', label: t('page.addNote'), icon: <Box /> },
		...pages,
		{ type: 'separator' },
		{ type: 'checkbox', id: 'toggleMargins', label: t('page.showMargins'), checked: marks.margins },
		{
			type: 'checkbox',
			id: 'toggleGrid',
			label: t('page.showGrid'),
			checked: marks.columns || marks.baseline,
		},
		...(hide.length > 0 || editMaster ? [{ type: 'separator' } as const] : []),
		...(editMaster
			? [
					{
						type: 'action',
						id: `editMaster:${editMaster.id}`,
						label: t('page.editMaster', { name: editMaster.name }),
						icon: <Box />,
					} as MenuItem,
				]
			: []),
		...hide.map((item): MenuItem => ({
			type: 'action',
			id: `${item.hidden ? 'showMaster' : 'hideMaster'}:${item.page}:${item.frame}`,
			label: t(item.hidden ? 'page.showMaster' : 'page.hideMaster', { name: item.label }),
			icon: <Box />,
		})),
		// Only people who can edit masters can change a page's own copy. What the page hides already
		// has one.
		...(editMaster ? hide.filter((item) => !item.hidden) : []).map((item): MenuItem => ({
			type: 'action',
			id: `copyMaster:${item.page}:${item.frame}`,
			label: t('page.copyMaster', { name: item.label }),
			icon: <Box />,
		})),
	];
}
