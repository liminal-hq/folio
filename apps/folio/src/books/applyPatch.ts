// Applies a patch from Rust to the UI's copy of a book
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Patch } from '@liminal-hq/folio-protocol/generated/Patch';
import type { PatchOp } from '@liminal-hq/folio-protocol/generated/PatchOp';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';

function applyOp(book: Book, op: PatchOp): Book {
	switch (op.op) {
		case 'setSpread': {
			const at = book.spreads.findIndex((s) => s.id === op.spread.id);
			const spreads: Spread[] =
				at === -1
					? [...book.spreads, op.spread]
					: book.spreads.map((s, i) => (i === at ? op.spread : s));
			return { ...book, spreads };
		}
		case 'removeSpread':
			return { ...book, spreads: book.spreads.filter((s) => s.id !== op.id) };
		case 'setOrder': {
			// Rust names the order of every spread; one that is not named is not in the book.
			const byId = new Map(book.spreads.map((s) => [s.id, s]));
			const spreads = op.order.flatMap((id) => {
				const spread = byId.get(id);
				return spread ? [spread] : [];
			});
			return { ...book, spreads };
		}
		case 'setMasters': {
			// Rust leaves these out of the book when there are none, and so does the copy.
			const { masterPages: _pages, pageNumbers: _numbers, ...rest } = book;
			return {
				...rest,
				masters: op.ids,
				...(op.masters.length > 0 ? { masterPages: op.masters } : {}),
				...(op.numbers !== 'none' ? { pageNumbers: op.numbers } : {}),
			};
		}
		case 'setDetails': {
			// Rust leaves a missing subtitle out of the book, and so does the copy.
			const { subtitle: _subtitle, ...rest } = book;
			const d = op.details;
			return {
				...rest,
				title: d.title,
				...(d.subtitle !== null ? { subtitle: d.subtitle } : {}),
				authors: d.authors,
				metadata: { ...book.metadata, description: d.description, isbn: d.isbn },
			};
		}
		case 'setLanguage':
			return { ...book, language: op.language };
		case 'setPreflightIgnore': {
			// Rust leaves an empty list out of the book, and so does the copy.
			const { preflightIgnore: _ignore, ...rest } = book;
			return op.keys.length > 0 ? { ...rest, preflightIgnore: op.keys } : rest;
		}
		case 'setFonts': {
			// Rust leaves an empty list out of the book, and so does the copy.
			const { fonts: _fonts, ...rest } = book;
			return op.fonts.length > 0 ? { ...rest, fonts: op.fonts } : rest;
		}
		case 'setProofing': {
			// Rust leaves an empty list out of the book, and so does the copy.
			const { proofing: _proofing, ...rest } = book;
			return op.proofing.ignore.length > 0 ? { ...rest, proofing: op.proofing } : rest;
		}
		case 'setGeometry':
			return {
				...book,
				trim: op.trim,
				margins: op.margins,
				bleed: op.bleed,
				grid: op.grid,
			};
	}
}

/**
 * The book with `patch` applied, as a new object. Ops apply in order, and Rust lists the spreads
 * it set before the new order, so an added spread is already in the book when the order names it.
 */
export function applyPatch(book: Book, patch: Patch): Book {
	return patch.ops.reduce(applyOp, book);
}
