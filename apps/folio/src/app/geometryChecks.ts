// What the page geometry dialogs check before they ask for a change, as the session does
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';

/** Whether margins leave room for text on a page of `trim`. */
export function marginsFit(trim: { w: number; h: number }, margins: Book['margins']): boolean {
	return margins.inside + margins.outside < trim.w && margins.top + margins.bottom < trim.h;
}
