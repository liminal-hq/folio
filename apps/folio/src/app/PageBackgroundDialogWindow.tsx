// The Page background dialog as a window of its own, beside the page it changes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useRef } from 'react';
import { useBooks } from '../books/useBooks';
import { backgroundAssets } from '../canvas/backgrounds';
import { PageBackgroundDialog } from '../canvas/PageBackgroundDialog';
import { useAssetUrls } from '../canvas/useAssetUrls';
import type { DialogPayloads } from '../dialogs/dialogKinds';
import type { DialogSession } from '../dialogs/DialogHost';
import { useDialogBook } from '../dialogs/useDialogBook';
import { usePaintPresets } from './usePaintPresets';

/**
 * Each try is applied at once, so the main window shows it behind (and is not dimmed). Done keeps
 * the tries as one undo step; Cancel and Esc rewind the book to how many steps it could undo when
 * the window opened.
 */
export function PageBackgroundDialogWindow({
	payload,
	session,
}: {
	payload: DialogPayloads['PageBackground'];
	session: DialogSession;
}) {
	const { store, dialogs, client } = useBooks();
	const dialog = useDialogBook(session);
	const mark = useRef<number | null>(null);
	const loaded = dialog !== null;
	// The mark is taken once the book has been read, before the first try is applied.
	useEffect(() => {
		if (!loaded) return;
		let live = true;
		void store.mark().then((found) => {
			if (live && mark.current === null) mark.current = found;
		});
		return () => {
			live = false;
		};
	}, [store, loaded]);
	const book = dialog?.book ?? null;
	const paints = usePaintPresets(client, true);
	const images = useAssetUrls(client, payload.bookId, book ? backgroundAssets(book) : []);
	if (!dialog || !book) return null;
	const spread = book.spreads.find((s) => s.id === payload.spread);
	if (!spread) return null;
	return (
		<PageBackgroundDialog
			open
			onClose={dialog.close}
			onCancel={() => {
				const back = mark.current;
				if (back !== null) void store.rewind(back).then(() => session.close());
				else session.close();
			}}
			book={book}
			spread={spread}
			page={payload.page}
			scope={payload.scope}
			presets={paints}
			images={images}
			onPickPicture={async () => {
				const picked = await dialogs.pickImage();
				if (picked === null) return null;
				try {
					return await dialog.client.importImage(dialog.id, picked);
				} catch {
					return null;
				}
			}}
			onApply={dialog.apply}
		/>
	);
}
