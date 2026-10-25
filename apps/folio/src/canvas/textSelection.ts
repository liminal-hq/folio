// Which stretch of words is selected in the open text editor, for the ribbon's bold, italic and font
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** A selected stretch of a frame's words, counted in characters (not UTF-16 units). */
export interface TextSelection {
	frame: string;
	start: number;
	end: number;
}

let current: TextSelection | null = null;

/** The editor says what is selected, or `null` when it closes. */
export function setTextSelection(selection: TextSelection | null): void {
	current = selection;
}

/** What is selected now in the open editor, when something is. */
export function getTextSelection(): TextSelection | null {
	return current && current.end > current.start ? current : null;
}

/** The characters (code points) before UTF-16 position `index` of `text`. */
export function charsBefore(text: string, index: number): number {
	return [...text.slice(0, index)].length;
}
