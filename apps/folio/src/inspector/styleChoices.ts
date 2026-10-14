// The named styles a book offers, read from its `styles.json`
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

export interface StyleChoice {
	id: string;
	name: string;
	/** The size the style sets, in points, when it sets one. */
	size: number | null;
}

/** The styles in the book's style file, in the order the file lists them. */
export function styleChoices(styles: unknown): StyleChoice[] {
	const table = (styles as { styles?: Record<string, Record<string, unknown>> } | null)?.styles;
	if (!table) return [];
	return Object.entries(table).map(([id, def]) => ({
		id,
		name: typeof def.name === 'string' ? def.name : id,
		size: typeof def.sizePt === 'number' ? def.sizePt : null,
	}));
}
