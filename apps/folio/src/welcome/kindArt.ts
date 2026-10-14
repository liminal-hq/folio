// The two small pages each kind of book is drawn with on the welcome screen
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { BookKind } from '@liminal-hq/folio-protocol/generated/BookKind';

/** A page's look: a tint (a picture), or `ruled` for a page of text. */
export type PageArt = 'violet' | 'sky' | 'green' | 'orange' | 'rose' | 'ruled';

/** Left page and right page, as the wizard prototype draws them. */
export const KIND_ART: Record<BookKind, [PageArt, PageArt]> = {
	picture: ['violet', 'ruled'],
	novel: ['ruled', 'ruled'],
	textbook: ['ruled', 'green'],
	illustrated: ['orange', 'ruled'],
	poetry: ['ruled', 'ruled'],
	memoir: ['green', 'ruled'],
};
