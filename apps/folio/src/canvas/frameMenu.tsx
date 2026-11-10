// The frame context menu and the commands it and the keys share
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Surface } from '@liminal-hq/folio-protocol/generated/Surface';
import {
	BringToFrontIcon,
	CopyIcon,
	CutIcon,
	DuplicateIcon,
	PasteIcon,
	LockIcon,
	SendToBackIcon,
	TrashIcon,
	UnlockIcon,
} from '../icons/AppIcons';
import { t } from '../i18n/messages';

export type FrameCommand =
	'cut' | 'copy' | 'paste' | 'duplicate' | 'front' | 'back' | 'lock' | 'unlock' | 'delete';

/** The commands that act on the clipboard, which the book view answers, not the frames one by one. */
export const CLIPBOARD_COMMANDS: readonly FrameCommand[] = ['cut', 'copy', 'paste'];

/** The rows of the menu for the selected `frames` (the one pressed, or all of them). */
export function frameMenuItems(frames: Frame[]): MenuItem[] {
	const allLocked = frames.every((f) => f.locked);
	return [
		{
			type: 'action',
			id: 'cut',
			label: t('command.cut'),
			shortcut: 'Ctrl+X',
			icon: <CutIcon />,
			// A locked frame cannot come off the page, so there is nothing to cut.
			disabled: allLocked,
		},
		{
			type: 'action',
			id: 'copy',
			label: t('command.copy'),
			shortcut: 'Ctrl+C',
			icon: <CopyIcon />,
		},
		{
			type: 'action',
			id: 'paste',
			label: t('command.paste'),
			shortcut: 'Ctrl+V',
			icon: <PasteIcon />,
		},
		{ type: 'separator' },
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
export function frameIntents(command: FrameCommand, surface: Surface, frame: Frame): Intent[] {
	const on = { on: surface, frame: frame.id };
	switch (command) {
		case 'cut':
		case 'copy':
		case 'paste':
			return [];
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
