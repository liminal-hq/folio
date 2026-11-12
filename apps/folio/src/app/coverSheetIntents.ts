// The change that puts a book's description on the back cover, shared by the page and the dialog window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { newFrame } from '../canvas/newFrames';

/** The change that sets the back cover's words to `text`, or null when the book has no cover. */
export function putDescriptionIntent(book: Book, text: string): Intent | null {
	const cover = book.spreads[0];
	if (!cover) return null;
	if (cover.frames.some((f) => f.id === 'blurb')) return { type: 'SetText', frame: 'blurb', text };
	// The back cover's own page, from its top-left: half an inch in from each edge.
	const frame = newFrame(
		'text',
		{ x: 36, y: 54, w: book.trim.w - 72, h: book.trim.h - 144 },
		'body',
	);
	return {
		type: 'AddFrame',
		on: { spread: cover.id },
		frame: { ...frame, id: 'blurb', content: { inline: [{ text }] } } as unknown as Frame,
	};
}
