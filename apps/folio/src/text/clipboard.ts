// Copying words with their look: Folio's own format for Folio, HTML for other programs, plain text for the rest
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { InlineContent } from '@liminal-hq/folio-protocol/generated/InlineContent';
import type { Run } from '@liminal-hq/folio-protocol/generated/Run';
import type { StoryPara } from '@liminal-hq/folio-protocol/generated/StoryPara';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { cpLength, cpSlice } from './units';

/** The clipboard type that carries paragraphs of runs between Folio's text editors. */
export const FOLIO_TEXT_MIME = 'application/x-folio-text+json';

/** The runs of a frame's own words, one list for each paragraph. */
export function paragraphRuns(content: InlineContent | null | undefined): Run[][] {
	const paras: Run[][] = [[]];
	for (const run of content?.inline ?? []) {
		run.text.split('\n').forEach((piece, i) => {
			if (i > 0) paras.push([]);
			if (piece !== '') paras[paras.length - 1]!.push({ ...run, text: piece });
		});
	}
	return paras;
}

/** The paragraphs from `from` to `to`, with the look of every stretch. */
export function sliceParagraphs(
	content: InlineContent | null | undefined,
	from: TextPos,
	to: TextPos,
): StoryPara[] {
	const paras = paragraphRuns(content);
	const out: StoryPara[] = [];
	for (let p = from.para; p <= to.para; p += 1) {
		const runs = paras[p] ?? [];
		const total = runs.reduce((n, r) => n + cpLength(r.text), 0);
		const lo = p === from.para ? from.offset : 0;
		const hi = p === to.para ? to.offset : total;
		const inline: Run[] = [];
		let at = 0;
		for (const run of runs) {
			const len = cpLength(run.text);
			const a = Math.max(lo, at) - at;
			const b = Math.min(hi, at + len) - at;
			if (a < b) inline.push({ ...run, text: cpSlice(run.text, a, b) });
			at += len;
		}
		out.push({ style: null, inline });
	}
	return out;
}

const escapeHtml = (text: string) =>
	text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

/** The paragraphs as simple HTML for another program: a `<p>` each, bold, italic, underline, strike, colour and links kept. */
export function toHtml(paragraphs: readonly StoryPara[]): string {
	return paragraphs
		.map((p) => {
			const body = p.inline
				.map((run) => {
					let html = escapeHtml(run.text);
					if (run.colour) html = `<span style="color:${escapeHtml(run.colour)}">${html}</span>`;
					if (run.bold) html = `<strong>${html}</strong>`;
					if (run.italic) html = `<em>${html}</em>`;
					if (run.strike) html = `<s>${html}</s>`;
					if (run.underline) html = `<u>${html}</u>`;
					if (run.href)
						html = `<a href="${escapeHtml(run.href).replace(/"/g, '&quot;')}">${html}</a>`;
					return html;
				})
				.join('');
			return `<p>${body}</p>`;
		})
		.join('');
}

/** The paragraphs as plain text, a line break between them. */
export function toPlain(paragraphs: readonly StoryPara[]): string {
	return paragraphs.map((p) => p.inline.map((r) => r.text).join('')).join('\n');
}

/** Paragraphs read back from Folio's clipboard type, or `null` when it holds anything else. */
export function fromClipboard(json: string): StoryPara[] | null {
	try {
		const parsed: unknown = JSON.parse(json);
		if (!Array.isArray(parsed) || parsed.length === 0) return null;
		const ok = parsed.every(
			(p) =>
				typeof p === 'object' &&
				p !== null &&
				Array.isArray((p as StoryPara).inline) &&
				(p as StoryPara).inline.every((r) => typeof r?.text === 'string'),
		);
		return ok ? (parsed as StoryPara[]) : null;
	} catch {
		return null;
	}
}

/** The words last copied in a Folio editor, so a paste that cannot read Folio's type keeps their look. */
let copied: { plain: string; paragraphs: StoryPara[] | null } | null = null;

const sameWords = (a: string, b: string) => a.replace(/\r\n?/g, '\n') === b.replace(/\r\n?/g, '\n');

/** Remembers the words a Folio editor just copied, with their look when they have one. */
export function rememberCopied(plain: string, paragraphs: StoryPara[] | null): void {
	copied = { plain, paragraphs };
}

/** The look of the words last copied in Folio, when the clipboard still holds those words. */
export function copiedLook(plain: string): StoryPara[] | null {
	return copied && copied.paragraphs && sameWords(copied.plain, plain) ? copied.paragraphs : null;
}

/** The words last copied in Folio, for when the clipboard cannot be read at all. */
export function lastCopied(): string | null {
	return copied?.plain ?? null;
}
