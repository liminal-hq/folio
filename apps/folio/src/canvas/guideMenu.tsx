// The menu of a ruler guide: lock it, delete it, or clear guides
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import type { Guide } from '@liminal-hq/folio-protocol/generated/Guide';
import { t } from '../i18n/messages';
import type { GuideCommand } from './guides';

const Mark = () => (
	<svg
		width="16"
		height="16"
		viewBox="0 0 16 16"
		fill="none"
		stroke="currentColor"
		strokeWidth="1.3"
		aria-hidden="true"
	>
		<path d="M8 2v12" />
	</svg>
);

export function guideMenuItems(guide: Guide): MenuItem[] {
	return [
		{
			type: 'action',
			id: guide.locked ? 'unlock' : 'lock',
			label: t(guide.locked ? 'guide.unlock' : 'guide.lock'),
			icon: <Mark />,
		},
		{ type: 'separator' },
		{ type: 'action', id: 'clearSpread', label: t('guide.clearSpread'), icon: <Mark /> },
		{ type: 'action', id: 'clearAll', label: t('guide.clearAll'), icon: <Mark /> },
		{ type: 'separator' },
		{
			type: 'action',
			id: 'delete',
			label: t('guide.delete'),
			icon: <Mark />,
			danger: true,
			disabled: guide.locked,
		},
	];
}

export const isGuideCommand = (id: string): id is GuideCommand =>
	['lock', 'unlock', 'delete', 'clearSpread', 'clearAll'].includes(id);
