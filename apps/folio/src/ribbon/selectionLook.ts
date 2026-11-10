// The words under the open editor's caret or selection, and the look they have, for the ribbon
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { InlineContent } from '@liminal-hq/folio-protocol/generated/InlineContent';
import type { Run } from '@liminal-hq/folio-protocol/generated/Run';
import type { RunChange } from '@liminal-hq/folio-protocol/generated/RunChange';
import type { StoryPara } from '@liminal-hq/folio-protocol/generated/StoryPara';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import type { TextTarget } from '@liminal-hq/folio-protocol/generated/TextTarget';
import type { TextCaret, TextSelection } from '../canvas/textSelection';

/** What a look is read from: the open editor's selection and caret, and the frames and stories. */
export interface LookInput {
	selected: Frame[];
	selection?: TextSelection | null;
	caret?: TextCaret | null;
	flows?: readonly { id: string; paragraphs: readonly StoryPara[] }[];
}

/** Words the ribbon can style: where they are, how they are divided and what is selected in them. */
export interface LookScope {
	/** The frame the editor was opened on: a text frame, a shape or a table. */
	frame: Frame;
	/** Whose words, for the words of a story or a table cell; none for a frame's or a shape's own. */
	target: Extract<TextTarget, { type: 'story' | 'cell' }> | null;
	/** The runs of each paragraph. */
	paras: readonly (readonly Run[])[];
	from: TextPos;
	to: TextPos;
	/** Nothing is selected: the caret is at `from`. */
	collapsed: boolean;
}

/** A frame's or cell's words as the runs of each paragraph (a line break ends one). */
export function paragraphRuns(content: InlineContent | null | undefined): Run[][] {
	const paras: Run[][] = [[]];
	for (const run of content?.inline ?? []) {
		run.text.split('\n').forEach((piece, i) => {
			if (i > 0) paras.push([]);
			paras[paras.length - 1]!.push({ ...run, text: piece });
		});
	}
	return paras;
}

const usable = (f: Frame | undefined) => f !== undefined && !f.locked && !f.fromMaster;

/** The words the editor's selection, or its caret when nothing is selected, is in, if the ribbon may style them. */
export function lookScope(input: LookInput): LookScope | null {
	const { selection, caret } = input;
	const range = selection && selection.end > selection.start ? selection : null;
	const at = range ?? caret;
	if (!at) return null;
	const frame = input.selected.find((f) => f.id === at.frame);
	if (!usable(frame)) return null;
	const target = at.target;
	const from = range ? range.from : caret?.pos;
	const to = range ? range.to : caret?.pos;
	if (target?.type === 'story') {
		if (frame!.type !== 'text' || !from || !to) return null;
		const story = input.flows?.find((f) => f.id === target.flow);
		if (!story) return null;
		return {
			frame: frame!,
			target,
			paras: story.paragraphs.map((p) => p.inline),
			from,
			to,
			collapsed: !range,
		};
	}
	if (target?.type === 'cell') {
		if (frame!.type !== 'table' || !from || !to) return null;
		const cell = frame!.rows[target.row]?.[target.col];
		if (!cell) return null;
		return {
			frame: frame!,
			target,
			paras: paragraphRuns(cell.content),
			from,
			to,
			collapsed: !range,
		};
	}
	if (frame!.type !== 'text' && frame!.type !== 'shape') return null;
	return {
		frame: frame!,
		target: null,
		paras: [frame!.content?.inline ?? []],
		from: { para: 0, offset: range ? range.start : caret!.offset },
		to: { para: 0, offset: range ? range.end : caret!.offset },
		collapsed: !range,
	};
}

/** The runs with at least one character between `from` and `to`. */
export function runsOver(scope: LookScope): Run[] {
	const out: Run[] = [];
	for (let p = scope.from.para; p <= scope.to.para; p += 1) {
		const start = p === scope.from.para ? scope.from.offset : 0;
		const end = p === scope.to.para ? scope.to.offset : Infinity;
		let at = 0;
		for (const run of scope.paras[p] ?? []) {
			const length = [...run.text].length;
			if (length > 0 && at < end && at + length > start) out.push(run);
			at += length;
		}
	}
	return out;
}

/** The run a typed character would take its look from: the one before the caret, or the paragraph's first. */
export function runBeforeCaret(scope: LookScope): Run | undefined {
	const runs = scope.paras[scope.from.para] ?? [];
	let at = 0;
	for (const run of runs) {
		const length = [...run.text].length;
		if (scope.from.offset > at && scope.from.offset <= at + length) return run;
		at += length;
	}
	return scope.from.offset === 0 ? runs[0] : runs[runs.length - 1];
}

/** What the ribbon shows of the words the scope covers: set only when every character agrees. */
export interface ScopeLook {
	bold: boolean;
	italic: boolean;
	underline: boolean;
	strike: boolean;
	/** The size of the first character in points, and whether the characters differ in size. */
	size: number;
	sizeMixed: boolean;
	/** The letter spacing they share in em thousandths (`undefined`: the style's), and whether they differ. */
	tracking: number | undefined;
	trackingMixed: boolean;
	/** The one language they share ('' when it is the book's), or `null` when they differ. */
	lang: string | null;
	/** The one typeface they share ('' when it is the style's), or `null` when they differ. */
	font: string | null;
}

/** The look of the selected words: bold, italic, underline and strike only when every character has them, and the font when they share one. */
export function lookOf(scope: LookScope, sizeOf: (run: Run) => number = () => 12): ScopeLook {
	const runs = scope.collapsed
		? [runBeforeCaret(scope)].filter((r): r is Run => !!r)
		: runsOver(scope);
	if (runs.length === 0)
		return {
			bold: false,
			italic: false,
			underline: false,
			strike: false,
			size: 12,
			sizeMixed: false,
			tracking: undefined,
			trackingMixed: false,
			lang: '',
			font: '',
		};
	const fonts = new Set(runs.map((r) => r.font ?? ''));
	const langs = new Set(runs.map((r) => r.lang ?? ''));
	const sizes = new Set(runs.map(sizeOf));
	const trackings = new Set(runs.map((r) => r.tracking ?? null));
	return {
		bold: runs.every((r) => r.bold),
		italic: runs.every((r) => r.italic),
		underline: runs.every((r) => r.underline === true),
		strike: runs.every((r) => r.strike === true),
		size: sizeOf(runs[0]!),
		sizeMixed: sizes.size > 1,
		tracking: trackings.size === 1 ? ([...trackings][0] ?? undefined) : undefined,
		trackingMixed: trackings.size > 1,
		lang: langs.size === 1 ? [...langs][0]! : null,
		font: fonts.size === 1 ? [...fonts][0]! : null,
	};
}

/** The intent that changes the look of the scope's selected words. */
export function styleIntent(scope: LookScope, change: RunChange): Intent {
	return scope.target
		? { type: 'StyleText', target: scope.target, from: scope.from, to: scope.to, change }
		: {
				type: 'StyleRange',
				frame: scope.frame.id,
				start: scope.from.offset,
				end: scope.to.offset,
				change,
			};
}

/** The link the caret is in or the selection touches, and the words a new link would be set on. */
export interface LinkTarget {
	/** Whether the caret or selection is on an existing link. */
	link: boolean;
	/** The address they share ('' when there is none, or when they link to different places). */
	href: string;
	/** The selection touches links to different places. */
	mixed: boolean;
	/** The words the link covers: a link the caret is in whole, a selection with its links whole. */
	from: TextPos;
	to: TextPos;
	/** Whether any words are there to link: a caret that is not in a link has none. */
	words: boolean;
}

const lengthOf = (run: Run) => [...run.text].length;

/** Where each run of a paragraph begins and ends, in characters. */
function spansOf(runs: readonly Run[]) {
	let at = 0;
	return runs.map((run) => {
		const span = { run, start: at, end: at + lengthOf(run) };
		at = span.end;
		return span;
	});
}

/**
 * What a link would be set on or edited for the caret or selection of `scope`. A caret in a link is
 * on all of it (a link runs from one run to the next that is not it); a selection that touches one
 * link only takes the rest of that link in with it, so the address is changed for all its words.
 */
export function linkTarget(scope: LookScope): LinkTarget {
	if (scope.collapsed) {
		const { para, offset } = scope.from;
		const near = spansOf(scope.paras[para] ?? []).filter(
			(s) => s.run.href && s.end > s.start && offset >= s.start && offset <= s.end,
		);
		const on = near.find((s) => offset > s.start && offset < s.end) ?? near[0];
		if (on)
			return {
				link: true,
				href: on.run.href!,
				mixed: false,
				from: { para, offset: on.start },
				to: { para, offset: on.end },
				words: true,
			};
		return { link: false, href: '', mixed: false, from: scope.from, to: scope.to, words: false };
	}
	const hrefs = [
		...new Set(
			runsOver(scope)
				.map((r) => r.href ?? '')
				.filter((h) => h !== ''),
		),
	];
	let { from, to } = scope;
	if (hrefs.length === 1) {
		const href = hrefs[0]!;
		const reach = (pos: TextPos, side: 'start' | 'end'): TextPos => {
			const hit = spansOf(scope.paras[pos.para] ?? []).find(
				(s) => s.run.href === href && pos.offset > s.start && pos.offset < s.end,
			);
			return hit ? { para: pos.para, offset: side === 'start' ? hit.start : hit.end } : pos;
		};
		from = reach(from, 'start');
		to = reach(to, 'end');
	}
	return {
		link: hrefs.length > 0,
		href: hrefs.length === 1 ? hrefs[0]! : '',
		mixed: hrefs.length > 1,
		from,
		to,
		words: true,
	};
}

/** The intent that sets, changes or (with an empty `href`) takes away the link on `target`. */
export function linkIntent(scope: LookScope, target: LinkTarget, href: string): Intent {
	return styleIntent({ ...scope, from: target.from, to: target.to, collapsed: false }, { href });
}

/** The words between two places of the scope, a line break between paragraphs, cut short to `max` characters. */
export function textBetween(scope: LookScope, from: TextPos, to: TextPos, max = 60): string {
	const pieces: string[] = [];
	for (let p = from.para; p <= to.para; p += 1) {
		const chars = [...(scope.paras[p] ?? []).map((r) => r.text).join('')];
		pieces.push(
			chars
				.slice(p === from.para ? from.offset : 0, p === to.para ? to.offset : undefined)
				.join(''),
		);
	}
	const text = pieces.join(' ').replace(/\s+/g, ' ').trim();
	const chars = [...text];
	return chars.length > max ? `${chars.slice(0, max - 1).join('')}…` : text;
}
