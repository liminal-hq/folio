// Blob addresses for the pictures a spread shows, read from the book once each
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useRef, useState } from 'react';
import type { BooksClient } from '../books/booksClient';

const TYPES: Record<string, string> = {
	png: 'image/png',
	jpg: 'image/jpeg',
	jpeg: 'image/jpeg',
	gif: 'image/gif',
	webp: 'image/webp',
	svg: 'image/svg+xml',
};

/** The media type a picture's path says it is. */
export function mediaTypeOf(asset: string): string {
	return TYPES[asset.split('.').pop()?.toLowerCase() ?? ''] ?? 'application/octet-stream';
}

/**
 * An address the page can draw for each asset path asked for, by path. A picture is read from the
 * book once while the book is shown, and its address is let go when the book changes or the page
 * closes.
 */
export function useAssetUrls(
	client: BooksClient,
	bookId: string,
	assets: string[],
): Record<string, string> {
	const [held, setHeld] = useState<{ book: string; urls: Record<string, string> }>({
		book: bookId,
		urls: {},
	});
	// What has been asked for and made, per book. A ref, because it is the effect's own bookkeeping.
	const state = useRef<{
		book: string;
		urls: Map<string, string>;
		asked: Set<string>;
		closed: boolean;
	} | null>(null);

	const wanted = [...new Set(assets)].sort().join('\n');
	useEffect(() => {
		let current = state.current;
		if (current === null || current.book !== bookId) {
			current?.urls.forEach((url) => URL.revokeObjectURL(url));
			current = { book: bookId, urls: new Map(), asked: new Set(), closed: false };
			state.current = current;
			setHeld({ book: bookId, urls: {} });
		}
		const mine = current;
		for (const asset of wanted ? wanted.split('\n') : []) {
			if (mine.asked.has(asset)) continue;
			mine.asked.add(asset);
			client
				.assetData(bookId, asset)
				.then((bytes) => {
					// The book changed, or the page closed, while the picture was being read.
					if (mine.closed || state.current !== mine) return;
					const url = URL.createObjectURL(new Blob([bytes], { type: mediaTypeOf(asset) }));
					mine.urls.set(asset, url);
					setHeld({ book: bookId, urls: Object.fromEntries(mine.urls) });
				})
				.catch(() => {
					// Missing from the book for now: the frame keeps its placeholder, and may ask again.
					mine.asked.delete(asset);
				});
		}
	}, [client, bookId, wanted]);

	useEffect(
		() => () => {
			const current = state.current;
			if (!current) return;
			current.closed = true;
			current.urls.forEach((url) => URL.revokeObjectURL(url));
			state.current = null;
		},
		[],
	);

	return held.book === bookId ? held.urls : NONE;
}

const NONE: Record<string, string> = {};
