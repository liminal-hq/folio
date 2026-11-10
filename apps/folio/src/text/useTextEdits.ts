// Sending the editor's edits to the book, and keeping the layout that comes back with each
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import type { SmartNote } from '@liminal-hq/folio-protocol/generated/SmartNote';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { useCallback, useState } from 'react';
import type { BooksStore } from '../books/booksStore';

/** A layout that came back with an edit: the spread at the revision the edit made. */
export interface LayoutSeed {
	spread: string;
	revision: number;
	layout: SpreadLayout;
}

/**
 * The function the editor makes its edits with, and the layout each one brings back, for the
 * window to draw at once instead of asking for the spread again.
 */
export function useTextEdits(
	store: BooksStore,
	spread: string | null,
	spelling: readonly string[] | null,
	smartPunctuation = true,
): {
	onEdit(edit: EditRequest): Promise<{ caret: TextPos; smart?: SmartNote }>;
	seed: LayoutSeed | null;
} {
	const [seed, setSeed] = useState<LayoutSeed | null>(null);
	const key = spelling === null ? null : spelling.join('\u0000');
	const onEdit = useCallback(
		async (edit: EditRequest) => {
			if (spread === null) throw new Error('there is no spread to edit');
			// The editor asks for smart punctuation on what is typed; the person's setting decides.
			const { smart, ...asked } = edit;
			const sent: EditRequest = smart && smartPunctuation ? { ...asked, smart: true } : asked;
			const answer = await store.editText(sent, spread, spelling ? [...spelling] : undefined, (a) =>
				setSeed({ spread, revision: a.layout.revision, layout: a.layout.layout }),
			);
			if (!answer) throw new Error('the book is closed');
			return { caret: answer.caret, ...(answer.smart ? { smart: answer.smart } : {}) };
		},
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[store, spread, key, smartPunctuation],
	);
	return { onEdit, seed };
}
