// The menu of ruler units, from the corner button or a right-click on a ruler
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import type { RulerUnit } from '@liminal-hq/folio-protocol/generated/RulerUnit';
import { t } from '../i18n/messages';
import { RULER_UNIT_ORDER } from './rulerMath';

export function rulerUnitItems(unit: RulerUnit): MenuItem[] {
	return [
		{ type: 'section', label: t('ruler.units') },
		...RULER_UNIT_ORDER.map((each): MenuItem => ({
			type: 'checkbox',
			id: each,
			label: t(`ruler.unit.${each}`),
			checked: each === unit,
		})),
	];
}
