// The shelf's thumbnail menu and the intents its commands make
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { DuplicateIcon, MoveLeftIcon, MoveRightIcon, TrashIcon } from '../icons/AppIcons';
import { t } from '../i18n/messages';

export type ShelfCommand = 'moveLeft' | 'moveRight' | 'duplicate' | 'remove';

/** The rows for the spread at `index`: the cover stays first and cannot be moved, copied or removed. */
export function shelfMenuItems(book: Book, index: number): MenuItem[] {
	const spread = book.spreads[index];
	const isCover = spread?.kind === 'cover';
	const last = book.spreads.length - 1;
	return [
		{
			type: 'action',
			id: 'moveLeft',
			label: t('shelf.moveLeft'),
			icon: <MoveLeftIcon />,
			disabled: isCover || index <= 1,
		},
		{
			type: 'action',
			id: 'moveRight',
			label: t('shelf.moveRight'),
			icon: <MoveRightIcon />,
			disabled: isCover || index >= last,
		},
		{
			type: 'action',
			id: 'duplicate',
			label: t('shelf.duplicate'),
			icon: <DuplicateIcon />,
			disabled: isCover,
		},
		{ type: 'separator' },
		{
			type: 'action',
			id: 'remove',
			label: t('shelf.remove'),
			danger: true,
			icon: <TrashIcon />,
			disabled: isCover || book.spreads.length <= 1,
		},
	];
}

/** The intent a shelf command makes, or `null` when it would do nothing. */
export function shelfIntent(command: ShelfCommand, book: Book, index: number): Intent | null {
	const spread = book.spreads[index];
	if (!spread || spread.kind === 'cover') return null;
	switch (command) {
		case 'moveLeft':
			return index > 1 ? { type: 'MoveSpread', spread: spread.id, to: index - 1 } : null;
		case 'moveRight':
			return index < book.spreads.length - 1
				? { type: 'MoveSpread', spread: spread.id, to: index + 1 }
				: null;
		case 'duplicate':
			return { type: 'AddSpread', after: spread.id, duplicate: true };
		case 'remove':
			return book.spreads.length > 1 ? { type: 'DeleteSpread', spread: spread.id } : null;
	}
}
