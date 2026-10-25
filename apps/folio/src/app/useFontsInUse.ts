// The typefaces a book uses, read from the plugin and read again when its fonts or styles change
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FontUse } from '@liminal-hq/folio-protocol/generated/FontUse';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';

/** The uses for the book `id`, or `undefined` until read; `revision` is when to ask again. */
export function useFontsInUse(
	client: BooksClient,
	id: string,
	revision: number,
	wanted: boolean,
): FontUse[] | undefined {
	const [uses, setUses] = useState<FontUse[] | undefined>(undefined);
	useEffect(() => {
		if (!wanted) return;
		let live = true;
		client
			.fontsInUse(id)
			.then((found) => live && setUses(found))
			.catch(() => undefined);
		return () => {
			live = false;
		};
	}, [client, id, revision, wanted]);
	return uses;
}
