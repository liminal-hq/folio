// The frame context menu and the commands it and the keys share
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import {
	BringToFrontIcon,
	DuplicateIcon,
	LockIcon,
	SendToBackIcon,
	TrashIcon,
	UnlockIcon,
} from '../icons/AppIcons';
import { t } from '../i18n/messages';

export type FrameCommand = 'duplicate' | 'front' | 'back' | 'lock' | 'unlock' | 'delete';

/** The rows of the menu for the selected `frames` (the one pressed, or all of them). */
export function frameMenuItems(frames: Frame[]): MenuItem[] {
	const allLocked = frames.every((f) => f.locked);
	return [
		{
			type: 'action',
			id: 'duplicate',
			label: t('frame.duplicate'),
			shortcut: 'Ctrl+D',
			icon: <DuplicateIcon />,
		},
		{ type: 'action', id: 'front', label: t('frame.front'), icon: <BringToFrontIcon /> },
		{ type: 'action', id: 'back', label: t('frame.back'), icon: <SendToBackIcon /> },
		{
			type: 'action',
			id: allLocked ? 'unlock' : 'lock',
			label: allLocked ? t('frame.unlock') : t('frame.lock'),
			icon: allLocked ? <UnlockIcon /> : <LockIcon />,
		},
		{ type: 'separator' },
		{
			type: 'action',
			id: 'delete',
			label: t('frame.delete'),
			shortcut: 'Del',
			danger: true,
			disabled: allLocked,
			icon: <TrashIcon />,
		},
	];
}

/** The intents a command makes for one frame. A locked frame is not deleted. */
export function frameIntents(command: FrameCommand, spread: string, frame: Frame): Intent[] {
	const on = { spread, frame: frame.id };
	switch (command) {
		case 'duplicate':
			return [{ type: 'DuplicateFrame', ...on }];
		case 'front':
			return [{ type: 'Reorder', ...on, to: 'front' }];
		case 'back':
			return [{ type: 'Reorder', ...on, to: 'back' }];
		case 'lock':
			return [{ type: 'LockFrame', ...on, locked: true }];
		case 'unlock':
			return [{ type: 'LockFrame', ...on, locked: false }];
		case 'delete':
			return frame.locked ? [] : [{ type: 'DeleteFrame', ...on }];
	}
}
