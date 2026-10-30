// Where each file the Export dialog makes goes, told from the name it was given
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MessageKey } from '../i18n/messages';

// The ends of the names `folio-export` gives its files, most specific first: `-fixed.fxl.kepub.epub`
// must be told apart from `-fixed.epub`, and `.kepub.epub` from `.epub`.
const GUIDE: readonly (readonly [string, MessageKey])[] = [
	['-fixed.fxl.kepub.epub', 'export.where.fixedKobo'],
	['-fixed.epub', 'export.where.fixedEpub'],
	['-kindle.epub', 'export.where.kindleEpub'],
	['.kepub.epub', 'export.where.kepub'],
	['.epub', 'export.where.koboEpub'],
	['-ebook-cover.jpg', 'export.where.ebookCover'],
	['-paperback-interior.pdf', 'export.where.kdpInterior'],
	['-paperback-cover.pdf', 'export.where.kdpCover'],
	['-print-pdfx4.pdf', 'export.where.printShop'],
	['-share.pdf', 'export.where.sharing'],
];

/** What the file called `name` is for and where to put it, or `null` for a name Folio did not give. */
export function whereItGoes(name: string): MessageKey | null {
	const lower = name.toLowerCase();
	return GUIDE.find(([end]) => lower.endsWith(end))?.[1] ?? null;
}
