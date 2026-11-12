// The ribbon's Paragraph group: which paragraphs a control acts on, what it shows and what it asks for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { ParaChange } from '@liminal-hq/folio-protocol/generated/ParaChange';
import type { ParaField } from '@liminal-hq/folio-protocol/generated/ParaField';
import type { ParaLook } from '@liminal-hq/folio-protocol/generated/ParaLook';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import type { StoryPara } from '@liminal-hq/folio-protocol/generated/StoryPara';
import type { TextTarget } from '@liminal-hq/folio-protocol/generated/TextTarget';
import type { TextParagraphs } from '../canvas/textSelection';
import { t } from '../i18n/messages';
import type { StyleChoice } from '../inspector/styleChoices';
import { effectiveSize } from '../inspector/textFacts';
import { paragraphsOf } from '../text/edit';

/** The line spacings the menu offers, as a multiple of the type size. */
export const LINE_SPACINGS = [1, 1.15, 1.3, 1.5, 1.8, 2] as const;
/** The spaces before and after a paragraph the menus offer, in points. */
export const PARAGRAPH_SPACES = [0, 4, 8, 12, 18, 24] as const;
/** How far Indent and Outdent move a paragraph, in points (a quarter of an inch). */
export const INDENT_STEP = 18;
/** The first-line indent the button gives, as a multiple of the type size. */
export const FIRST_INDENT_EM = 1.2;
/** The most an indent may be, which Rust also holds to. */
export const MAX_INDENT = 720;

/** The value a menu sends to take a paragraph's own setting away, so its style's shows again. */
export const FROM_STYLE = 'style';

/** What the controls need of the selection and the book to find their paragraphs. */
export interface ParagraphInput {
	selected: Frame[];
	layout: SpreadLayout | null;
	choices: StyleChoice[];
	/** The paragraphs the open editor's caret or selection is in. */
	paragraphs?: TextParagraphs | null;
	/** The stories of the book, for the looks of the paragraphs of a flowing frame. */
	flows?: readonly { id: string; paragraphs: readonly StoryPara[] }[];
}

/** One run of paragraphs some words have, to change together. */
export interface ParagraphScope {
	target: TextTarget;
	from: number;
	to: number;
	/** The look of each paragraph `from..to`. */
	looks: ParaLook[];
	/** The size the words are set in, which a first-line indent is a multiple of. */
	size: number;
	/** What the frame says for its whole box, under what the paragraphs set. */
	align: 'left' | 'center' | 'right' | 'justify';
}

const slice = (looks: readonly ParaLook[] | undefined, from: number, to: number): ParaLook[] =>
	Array.from({ length: to - from + 1 }, (_, i) => looks?.[from + i] ?? {});

/**
 * The paragraphs the controls act on: those the open editor's caret or selection is in; and when no
 * editor is open, every paragraph of each selected frame that has words of its own (a frame of a
 * story, the paragraphs it shows).
 */
export function paragraphScopes(input: ParagraphInput): ParagraphScope[] {
	const free = input.selected.filter((f) => !f.locked && !f.fromMaster);
	const sizeOf = (f: Frame) =>
		f.type === 'text' ? effectiveSize(f, input.layout, input.choices) : 12;
	const alignOf = (f: Frame) => (f.type === 'text' ? (f.overrides?.align ?? 'left') : 'left');
	const open = input.paragraphs;
	if (open) {
		const frame = free.find((f) => f.id === open.frame);
		if (frame) {
			return [
				{
					target: open.target,
					from: open.from,
					to: open.to,
					looks: slice(open.looks, open.from, open.to),
					size: sizeOf(frame),
					align: alignOf(frame),
				},
			];
		}
	}
	const out: ParagraphScope[] = [];
	for (const frame of free) {
		if (frame.type === 'text' && frame.flow) {
			const id = frame.flow.id;
			const shown = input.layout?.frames.find((f) => f.frame === frame.id)?.lines;
			const numbers = (shown ?? []).flatMap((l) => (l.paragraph == null ? [] : [l.paragraph]));
			if (numbers.length === 0) continue;
			const from = Math.min(...numbers);
			const to = Math.max(...numbers);
			const story = input.flows?.find((f) => f.id === id);
			out.push({
				target: { type: 'story', flow: id },
				from,
				to,
				looks: slice(
					story?.paragraphs.map((p) => p.look ?? {}),
					from,
					to,
				),
				size: sizeOf(frame),
				align: alignOf(frame),
			});
		} else if (frame.type === 'text' || (frame.type === 'shape' && frame.content)) {
			const count = paragraphsOf(frame.content).length;
			out.push({
				target: { type: frame.type === 'text' ? 'frame' : 'shape', frame: frame.id },
				from: 0,
				to: count - 1,
				looks: slice(frame.content?.looks, 0, count - 1),
				size: sizeOf(frame),
				align: alignOf(frame),
			});
		}
	}
	return out;
}

const round = (n: number) => Math.round(n * 100) / 100;

/** The value a control shows for the paragraphs: the first one's, so a mixed run reads as its start. */
function first(scopes: ParagraphScope[]): ParaLook {
	return scopes[0]?.looks[0] ?? {};
}

const markChosen = (chosen: boolean, label: string) => (chosen ? `✓ ${label}` : label);

/** Marks what the controls of the Paragraph group show. */
export function paragraphState(
	input: ParagraphInput,
	available: Set<string>,
	active: Record<string, boolean>,
	options: Record<string, ReadonlyArray<readonly [string, string, string?]>>,
): void {
	const scopes = paragraphScopes(input);
	if (scopes.length === 0) return;
	for (const id of [
		'alignL',
		'alignC',
		'alignR',
		'justify',
		'lineSpacing',
		'spaceBefore',
		'spaceAfter',
		'indentLess',
		'indentMore',
		'firstIndent',
	])
		available.add(id);
	const look = first(scopes);
	const align = look.align ?? scopes[0]!.align;
	active.alignL = align === 'left';
	active.alignC = align === 'center';
	active.alignR = align === 'right';
	active.justify = align === 'justify';
	active.firstIndent = (look.firstIndent ?? 0) > 0;
	const spacing = (
		own: number | undefined,
		values: readonly number[],
		text: (n: number) => string,
	) => [
		...values.map((n) => [String(n), markChosen(own === n, text(n))] as const),
		[FROM_STYLE, markChosen(own === undefined, t('ribbon.para.fromStyle'))] as const,
	];
	options.lineSpacing = spacing(look.lineHeight, LINE_SPACINGS, (n) =>
		n.toFixed(2).replace(/0$/, ''),
	);
	options.spaceBefore = spacing(look.spaceBefore, PARAGRAPH_SPACES, (n) => `${n} pt`);
	options.spaceAfter = spacing(look.spaceAfter, PARAGRAPH_SPACES, (n) => `${n} pt`);
}

const ALIGNS = {
	alignL: 'left',
	alignC: 'center',
	alignR: 'right',
	justify: 'justify',
} as const;

/**
 * One intent for each run of neighbouring paragraphs that ends up with the same change, so a step
 * that depends on where each paragraph was (Indent) is a single undo step when they agree.
 */
const intentsFor = (
	scopes: ParagraphScope[],
	change: (scope: ParagraphScope, look: ParaLook) => ParaChange,
): Intent[] =>
	scopes.flatMap((scope) => {
		const runs: { from: number; to: number; change: ParaChange }[] = [];
		scope.looks.forEach((look, i) => {
			const next = change(scope, look);
			const last = runs[runs.length - 1];
			if (last && JSON.stringify(last.change) === JSON.stringify(next)) last.to = scope.from + i;
			else runs.push({ from: scope.from + i, to: scope.from + i, change: next });
		});
		return runs.map((run): Intent => ({ type: 'SetParagraphLook', target: scope.target, ...run }));
	});

const setting = (set: ParaLook, clear: ParaField[] = []): ParaChange => ({ set, clear });

/** The intents a control of the Paragraph group asks for, or `null` when it is not one of them. */
export function paragraphIntents(
	id: string,
	value: string | number | boolean | undefined,
	input: ParagraphInput,
): Intent[] | null {
	const scopes = paragraphScopes(input);
	if (scopes.length === 0) return null;
	const whole = (change: ParaChange): Intent[] =>
		scopes.map((s) => ({
			type: 'SetParagraphLook',
			target: s.target,
			from: s.from,
			to: s.to,
			change,
		}));
	switch (id) {
		case 'alignL':
		case 'alignC':
		case 'alignR':
		case 'justify':
			return whole(setting({ align: ALIGNS[id] }));
		case 'lineSpacing':
			return value === FROM_STYLE
				? whole(setting({}, ['lineHeight']))
				: Number.isFinite(Number(value))
					? whole(setting({ lineHeight: Number(value) }))
					: null;
		case 'spaceBefore':
		case 'spaceAfter': {
			const field = id === 'spaceBefore' ? 'spaceBefore' : 'spaceAfter';
			if (value === FROM_STYLE) return whole(setting({}, [field]));
			const points = Number(value);
			return Number.isFinite(points) ? whole(setting({ [field]: points })) : null;
		}
		case 'indentLess':
		case 'indentMore': {
			const step = id === 'indentMore' ? INDENT_STEP : -INDENT_STEP;
			return intentsFor(scopes, (_scope, look) =>
				setting({ indent: Math.min(MAX_INDENT, Math.max(0, (look.indent ?? 0) + step)) }),
			);
		}
		case 'firstIndent': {
			// Turned on unless the first paragraph has one; the rest follow it, so they all agree.
			const on = (first(scopes).firstIndent ?? 0) > 0;
			return intentsFor(scopes, (scope) =>
				setting({ firstIndent: on ? 0 : round(FIRST_INDENT_EM * scope.size) }),
			);
		}
		default:
			return null;
	}
}
