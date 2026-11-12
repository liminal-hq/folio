// Reading and writing words on the system clipboard from a button, where there is no clipboard event
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { lastCopied } from './clipboard';

/** How long the webview's clipboard is waited for, so a webview that never answers is given up on. */
const READ_WAIT_MS = 1500;

/** `promise`, or a refusal when it has not settled in time (a webview that never answers). */
function soon<T>(promise: Promise<T>): Promise<T> {
	return Promise.race([
		promise,
		new Promise<T>((_, reject) => setTimeout(() => reject(new Error('timed out')), READ_WAIT_MS)),
	]);
}

/** What the clipboard holds as words: plain, and HTML from another program when it has some. */
export interface ClipboardWords {
	plain: string;
	html: string;
}

/**
 * Reads the clipboard for a paste from the ribbon when the webview would not send a paste event for
 * it. The webview's clipboard is asked (it has the HTML, and may ask the person first); when it
 * refuses, the words last copied in Folio are all there is. `null`: nothing could be read.
 */
export async function readClipboardWords(): Promise<ClipboardWords | null> {
	const clipboard = typeof navigator === 'undefined' ? undefined : navigator.clipboard;
	try {
		if (clipboard?.read) {
			let plain = '';
			let html = '';
			for (const item of await soon(clipboard.read())) {
				if (!plain && item.types.includes('text/plain'))
					plain = await (await item.getType('text/plain')).text();
				if (!html && item.types.includes('text/html'))
					html = await (await item.getType('text/html')).text();
			}
			return { plain, html };
		}
	} catch {
		// Refused: the next way.
	}
	try {
		if (clipboard?.readText) return { plain: await soon(clipboard.readText()), html: '' };
	} catch {
		// Refused: the next way.
	}
	const mine = lastCopied();
	return mine === null ? null : { plain: mine, html: '' };
}

/** Puts words on the clipboard from a button, when the webview would not send a copy event for it. */
export async function writeClipboardWords(plain: string, html: string | null): Promise<boolean> {
	const clipboard = typeof navigator === 'undefined' ? undefined : navigator.clipboard;
	if (clipboard?.write && typeof ClipboardItem !== 'undefined') {
		try {
			const parts: Record<string, Blob> = {
				'text/plain': new Blob([plain], { type: 'text/plain' }),
			};
			if (html) parts['text/html'] = new Blob([html], { type: 'text/html' });
			await clipboard.write([new ClipboardItem(parts)]);
			return true;
		} catch {
			// Refused: the next way.
		}
	}
	try {
		await clipboard?.writeText(plain);
		return clipboard !== undefined;
	} catch {
		return false;
	}
}
