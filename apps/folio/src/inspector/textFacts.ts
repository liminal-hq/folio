// What the inspector and the ribbon read about a text frame's look
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import type { TextFrame } from '@liminal-hq/folio-protocol/generated/TextFrame';
import type { StyleChoice } from './styleChoices';

/** The size the words are set in, in points to a tenth: an override, what was laid out, or the style's. */
export function effectiveSize(
	frame: TextFrame,
	layout: SpreadLayout | null,
	choices: StyleChoice[],
): number {
	const laid = layout?.frames.find((f) => f.frame === frame.id);
	const style = choices.find((c) => c.id === frame.style);
	return (
		Math.round((frame.overrides?.size ?? laid?.lines[0]?.runs[0]?.size ?? style?.size ?? 12) * 10) /
		10
	);
}

/** The overrides of a frame with `change` laid over them, every field present as the intent needs. */
export function withOverrides(
	frame: TextFrame,
	change: Partial<NonNullable<TextFrame['overrides']>>,
): NonNullable<TextFrame['overrides']> {
	return {
		size: frame.overrides?.size ?? null,
		align: frame.overrides?.align ?? null,
		bold: frame.overrides?.bold ?? null,
		italic: frame.overrides?.italic ?? null,
		...change,
	};
}
