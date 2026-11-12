// The on-page editor that is open, for the ribbon's Copy, Cut and Paste and to give the focus back to it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** What the ribbon can ask of the open editor. */
export interface OpenEditor {
	/** Copies the selected words; answers whether any were selected. */
	copy(): Promise<boolean>;
	/** Copies the selected words and takes them out; answers whether any were selected. */
	cut(): Promise<boolean>;
	/**
	 * Pastes what the clipboard holds over the selection, as Ctrl+V (or Ctrl+Shift+V with `plain`)
	 * does; answers whether there were words to paste.
	 */
	paste(plain: boolean): Promise<boolean>;
	/** Gives the focus back to the editor, after a ribbon list or menu took it. */
	focus(): void;
}

let open: OpenEditor | null = null;

/** The editor says it is open, or (with `null`, or when `editor` is the one open) that it closed. */
export function setOpenEditor(editor: OpenEditor | null, closing?: OpenEditor): void {
	if (editor === null && closing !== undefined && open !== closing) return;
	open = editor;
}

/** The open on-page editor, when there is one. */
export function openEditor(): OpenEditor | null {
	return open;
}
