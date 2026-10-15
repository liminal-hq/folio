// The colours and gradients the pickers offer, read from the plugin once, when first wanted
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { PaintPresets } from '@liminal-hq/folio-protocol/generated/PaintPresets';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';

/**
 * The palette, or `null` until it has been read. It is read the first time `wanted` is true (a
 * dialog is open, or a shape is selected) and kept for as long as the screen is shown, so the
 * pickers never ask twice.
 */
export function usePaintPresets(client: BooksClient, wanted: boolean): PaintPresets | null {
	const [paints, setPaints] = useState<PaintPresets | null>(null);
	useEffect(() => {
		if (!wanted || paints !== null) return;
		let live = true;
		client
			.paintPresets()
			.then((found) => live && setPaints(found))
			.catch(() => undefined);
		return () => {
			live = false;
		};
	}, [wanted, paints, client]);
	return paints;
}
