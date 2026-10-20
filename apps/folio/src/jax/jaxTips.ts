// What Jax says: a tip for what the person is doing, a general pool, and the steps of a guide
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MessageKey } from '../i18n/messages';

/** What the person is doing, which decides the tip (spec 07: the first match wins). */
export interface TipContext {
	/** A picture frame is selected. */
	picture: boolean;
	/** A text frame is selected. */
	text: boolean;
	/** The stage shows a Kindle or a Kobo. */
	reader: 'print' | 'kindle' | 'kobo';
	/** Whether the book flows (a novel) or is laid out page by page (a picture book). */
	flowing: boolean;
	/** The columns or baseline grid are drawn. */
	grid: boolean;
}

/** The tip for a context, as a message key. */
export function contextualTip(context: TipContext): MessageKey | null {
	if (context.picture) return 'jax.tip.picture';
	if (context.text) return 'jax.tip.text';
	if (context.reader !== 'print') {
		return context.flowing ? 'jax.tip.reflow' : 'jax.tip.fixedPreview';
	}
	if (context.grid) return 'jax.tip.grid';
	return null;
}

/** Tips for any time, which "Another tip" goes through in turn. */
export const GENERAL_TIPS: readonly MessageKey[] = [
	'jax.tip.general.1',
	'jax.tip.general.2',
	'jax.tip.general.3',
	'jax.tip.general.4',
];

/** What a step of the guide can ask the book window to do. */
export type GuideAction =
	| { kind: 'page'; index: number }
	| { kind: 'preview'; mode: 'print' | 'kindle' | 'kobo' }
	| { kind: 'check' }
	| { kind: 'export' }
	| { kind: 'details' }
	| { kind: 'import' }
	| { kind: 'coverSheet' }
	| { kind: 'none' };

export interface GuideStep {
	title: MessageKey;
	body: MessageKey;
	/** What "Show me" does. */
	action: GuideAction;
	/** The label of "Show me", when it is not the usual. */
	show?: MessageKey;
}

/** The walk for a book laid out page by page. */
export const FIXED_GUIDE: readonly GuideStep[] = [
	{
		title: 'jax.guide.fixed.1.title',
		body: 'jax.guide.fixed.1.body',
		action: { kind: 'page', index: 1 },
	},
	{
		title: 'jax.guide.fixed.2.title',
		body: 'jax.guide.fixed.2.body',
		action: { kind: 'details' },
	},
	{
		title: 'jax.guide.fixed.3.title',
		body: 'jax.guide.fixed.3.body',
		action: { kind: 'none' },
	},
	{
		title: 'jax.guide.fixed.4.title',
		body: 'jax.guide.fixed.4.body',
		action: { kind: 'check' },
	},
	{
		title: 'jax.guide.fixed.5.title',
		body: 'jax.guide.fixed.5.body',
		action: { kind: 'preview', mode: 'kindle' },
	},
	{
		title: 'jax.guide.fixed.6.title',
		body: 'jax.guide.fixed.6.body',
		action: { kind: 'export' },
	},
];

/** The walk for a book that flows. */
export const FLOWING_GUIDE: readonly GuideStep[] = [
	{
		title: 'jax.guide.flowing.1.title',
		body: 'jax.guide.flowing.1.body',
		action: { kind: 'details' },
	},
	{
		title: 'jax.guide.flowing.2.title',
		body: 'jax.guide.flowing.2.body',
		action: { kind: 'import' },
	},
	{
		title: 'jax.guide.flowing.3.title',
		body: 'jax.guide.flowing.3.body',
		action: { kind: 'preview', mode: 'kindle' },
	},
	{
		title: 'jax.guide.flowing.4.title',
		body: 'jax.guide.flowing.4.body',
		action: { kind: 'coverSheet' },
	},
	{
		title: 'jax.guide.flowing.5.title',
		body: 'jax.guide.flowing.5.body',
		action: { kind: 'check' },
	},
	{
		title: 'jax.guide.flowing.6.title',
		body: 'jax.guide.flowing.6.body',
		action: { kind: 'export' },
	},
];
