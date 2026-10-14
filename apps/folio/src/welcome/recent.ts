// How the recents list shows a book's path
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

export interface RecentEntry {
	path: string;
	/** The file's name without `.folio`. */
	name: string;
	/** The folder the book is in. */
	folder: string;
}

/** A path split into the name to show and the folder it is in, for `/` and `\` separators alike. */
export function recentEntry(path: string): RecentEntry {
	const cut = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
	const file = cut === -1 ? path : path.slice(cut + 1);
	const folder = cut === -1 ? '' : path.slice(0, cut);
	return { path, name: file.replace(/\.folio$/i, ''), folder };
}
