// The pieces of the reader preview that are plain functions: the devices, the steps, the document
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ReaderPreview } from '@liminal-hq/folio-protocol/generated/ReaderPreview';
import type { MessageKey } from '../i18n/messages';

export type ReaderDevice = 'kindle' | 'kobo';
export type ReaderTheme = 'white' | 'sepia' | 'dark';

/** A device's screen, in the CSS pixels the preview is drawn at (half the panel's pixels). */
export interface DeviceSpec {
	name: MessageKey;
	/** The panel, in pixels. */
	panel: { width: number; height: number };
	width: number;
	height: number;
}

export const DEVICES: Record<ReaderDevice, DeviceSpec> = {
	kindle: {
		name: 'reader.device.kindle',
		panel: { width: 1236, height: 1648 },
		width: 618,
		height: 824,
	},
	kobo: {
		name: 'reader.device.kobo',
		panel: { width: 1072, height: 1448 },
		width: 536,
		height: 724,
	},
};

/** The text sizes a reader steps through, each about 12% from the last, as a share of the default. */
export const TEXT_STEPS: readonly number[] = [0.8, 0.9, 1, 1.12, 1.25, 1.4, 1.57, 1.76];

export const THEMES: readonly ReaderTheme[] = ['white', 'sepia', 'dark'];

const COLOURS: Record<ReaderTheme, { background: string; text: string }> = {
	white: { background: '#ffffff', text: '#1c1b20' },
	sepia: { background: '#f4ecd8', text: '#433422' },
	dark: { background: '#16151a', text: '#d7d3cc' },
};

/** The `@font-face` rules of a document, as text, to lend to another. */
export function fontFaceRules(doc: Document): string {
	const out: string[] = [];
	for (const sheet of Array.from(doc.styleSheets)) {
		let rules: CSSRuleList;
		try {
			rules = sheet.cssRules;
		} catch {
			continue; // A sheet from another origin cannot be read.
		}
		for (const rule of Array.from(rules)) {
			if (rule instanceof CSSFontFaceRule) out.push(rule.cssText);
		}
	}
	return out.join('\n');
}

/** How many pages a document `scrollWidth` wide makes when each page is `page` wide. */
export function pageCount(scrollWidth: number, page: number): number {
	return Math.max(1, Math.ceil((scrollWidth - 1) / page));
}

interface Options {
	preview: ReaderPreview;
	faces: string;
	scale: number;
	theme: ReaderTheme;
	/** Set the book in the reader's own font rather than the book's. */
	readerFont: boolean;
}

const escape = (text: string) =>
	text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/"/g, '&quot;');

/**
 * The document the preview shows: the EPUB's chapters and CSS, set in columns one screen wide so
 * the page turns by moving sideways, with the pictures put in as addresses.
 */
export function readerDocument({ preview, faces, scale, theme, readerFont }: Options): string {
	const { background, text } = COLOURS[theme];
	let body = preview.chapters.map((c) => `<section>${c.body}</section>`).join('\n');
	for (const image of preview.images) {
		body = body.split(`../images/${image.name}`).join(image.dataUri);
	}
	const family = readerFont ? "body, p { font-family: 'Inter', sans-serif !important; }" : '';
	return `<!doctype html><html lang="${escape(preview.language)}"><head><meta charset="utf-8"/><title>${escape(preview.title)}</title><style>
${faces}
${preview.css}
${family}
html { height: 100%; overflow: hidden; background: ${background}; }
body { margin: 0; height: 100%; box-sizing: border-box; padding: 28px; font-size: ${(18 * scale).toFixed(2)}px; color: ${text}; background: ${background};
  column-width: calc(100vw - 56px); column-gap: 56px; column-fill: auto; }
section { break-inside: auto; }
img { max-height: 70vh; }
</style></head><body>${body}</body></html>`;
}
