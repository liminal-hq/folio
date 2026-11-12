// Where exported files go, kept as a book can carry it: beside the book when it can be
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

const slashes = (path: string) => path.replace(/\\/g, '/');
const isWindowsPath = (path: string) => /^[a-z]:\//i.test(path);

/** The folder a file is in, with forward slashes, or `null` for a path with none. */
function folderOf(file: string): string | null {
	const path = slashes(file);
	const at = path.lastIndexOf('/');
	return at <= 0 ? null : path.slice(0, at);
}

/**
 * `folder` as a book keeps it: relative to the folder the book's file is in when it is inside it
 * (so the book can be moved, or shared, without carrying the home folder's name), and as it is
 * otherwise. A folder that is the book's own folder is `.`.
 */
export function relativeToBook(folder: string, bookFile: string | null): string {
	const base = bookFile === null ? null : folderOf(bookFile);
	if (base === null) return folder;
	const path = slashes(folder).replace(/\/+$/, '');
	const same = isWindowsPath(base)
		? (a: string, b: string) => a.toLowerCase() === b.toLowerCase()
		: (a: string, b: string) => a === b;
	if (same(path, base)) return '.';
	if (same(path.slice(0, base.length + 1), `${base}/`)) return path.slice(base.length + 1);
	return folder;
}

/** `folder` as a full path: a relative one is taken from the book's folder. */
export function absoluteFor(folder: string, bookFile: string | null): string {
	const path = slashes(folder);
	const base = bookFile === null ? null : folderOf(bookFile);
	const absolute = path.startsWith('/') || isWindowsPath(path) || path.startsWith('//');
	if (absolute || base === null) return folder;
	return path === '.' ? base : `${base}/${path}`;
}
