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
	const [urls, setUrls] = useState<Record<string, string>>({});
	const held = useRef<{ book: string; urls: Map<string, string>; asked: Set<string> }>({
		book: bookId,
		urls: new Map(),
		asked: new Set(),
	});

	if (held.current.book !== bookId) {
		held.current.urls.forEach((url) => URL.revokeObjectURL(url));
		held.current = { book: bookId, urls: new Map(), asked: new Set() };
	}

	const wanted = [...new Set(assets)].sort().join('\n');
	useEffect(() => {
		const current = held.current;
		for (const asset of wanted ? wanted.split('\n') : []) {
			if (current.asked.has(asset)) continue;
			current.asked.add(asset);
			client
				.assetData(bookId, asset)
				.then((bytes) => {
					if (held.current !== current) return;
					const url = URL.createObjectURL(new Blob([bytes], { type: mediaTypeOf(asset) }));
					current.urls.set(asset, url);
					setUrls(Object.fromEntries(current.urls));
				})
				.catch(() => {
					// The picture is missing from the book: the frame keeps its placeholder.
				});
		}
	}, [client, bookId, wanted]);

	useEffect(
		() => () => {
			held.current.urls.forEach((url) => URL.revokeObjectURL(url));
		},
		[],
	);

	return held.current.book === bookId ? urls : {};
}
