// The frames the page menu adds: a text box, a picture placeholder and a note box
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { t } from '../i18n/messages';
import type { PlainRect } from './geometry';

export type NewFrameKind = 'text' | 'picture' | 'note' | 'rect' | 'round' | 'ellipse';

/** A frame id no other frame has: the `fr_` prefix and 12 hex digits. */
export function newFrameId(): string {
	const bytes = crypto.getRandomValues(new Uint8Array(6));
	return `fr_${Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')}`;
}

/** How big each new frame starts, in points. */
const SIZES: Record<NewFrameKind, { w: number; h: number }> = {
	text: { w: 200, h: 80 },
	picture: { w: 240, h: 180 },
	note: { w: 160, h: 100 },
	rect: { w: 140, h: 100 },
	round: { w: 140, h: 100 },
	ellipse: { w: 140, h: 100 },
};

/** The box for a new frame centred on `at`, kept inside the trim of `page`. */
export function boxAt(
	kind: NewFrameKind,
	at: { x: number; y: number },
	page: PlainRect,
): PlainRect {
	const { w, h } = SIZES[kind];
	const width = Math.min(w, page.w);
	const height = Math.min(h, page.h);
	const x = Math.min(Math.max(at.x - width / 2, page.x), page.x + page.w - width);
	const y = Math.min(Math.max(at.y - height / 2, page.y), page.y + page.h - height);
	return { x, y, w: width, h: height };
}

/** The frame to add; `style` is the book style a text box starts in. */
export function newFrame(kind: NewFrameKind, box: PlainRect, style: string): Frame {
	const base = { id: newFrameId(), box, rotation: 0, locked: false };
	if (kind === 'text') {
		return {
			...base,
			type: 'text',
			style,
			content: { inline: [{ text: t('frame.new.text') }] },
			columns: 1,
			baselineLock: false,
		} as unknown as Frame;
	}
	if (kind === 'picture') {
		return {
			...base,
			type: 'image',
			image: { asset: null, fit: 'fill', focus: [0.5, 0.5], crop: null },
			placeholderLabel: t('frame.new.picture'),
		} as unknown as Frame;
	}
	if (kind === 'note') {
		return {
			...base,
			type: 'shape',
			shape: 'rect',
			fill: '#fff3b0',
			style: null,
			description: t('frame.new.note'),
		} as unknown as Frame;
	}
	return {
		...base,
		type: 'shape',
		shape: kind === 'round' ? 'rounded' : kind,
		fill: '#c9c4b8',
		style: null,
	} as unknown as Frame;
}
