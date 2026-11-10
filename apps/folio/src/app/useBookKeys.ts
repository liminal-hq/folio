// The keys for the book commands: save, open, new, close, undo, redo, find and replace
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect } from 'react';

/** Which command a key combination runs, or `null` for none. Control is Command on a Mac. */
export function commandForKey(event: {
	key: string;
	ctrlKey: boolean;
	metaKey: boolean;
	shiftKey: boolean;
	altKey: boolean;
}): string | null {
	if (!(event.ctrlKey || event.metaKey) || event.altKey) return null;
	switch (event.key.toLowerCase()) {
		case 's':
			return event.shiftKey ? 'file.saveAs' : 'file.save';
		case 'o':
			return 'file.open';
		case 'n':
			return 'file.new';
		case 'e':
			return 'file.export';
		case 'w':
			return 'file.close';
		case 'z':
			return event.shiftKey ? 'edit.redo' : 'edit.undo';
		case 'y':
			return 'edit.redo';
		case 'f':
			return event.shiftKey ? null : 'edit.find';
		case 'h':
			return event.shiftKey ? null : 'edit.replace';
		default:
			return null;
	}
}

/** Whether typing here belongs to the field, so undo and redo are its own. */
function isEditable(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	return target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
}

/** The words being edited on the page: their input is a field, but find is for the book. */
function isPageEditor(target: EventTarget | null): boolean {
	return target instanceof HTMLElement && target.closest('[data-text-sink]') !== null;
}

/** Runs book commands from the keyboard. A command that is not available does nothing. */
export function useBookKeys(run: (id: string) => Promise<void>): void {
	useEffect(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			const id = commandForKey(event);
			if (id === null) return;
			// The field handles its own undo and redo.
			if ((id === 'edit.undo' || id === 'edit.redo') && isEditable(event.target)) return;
			// Find is the book's, except in another field (the find bar has its own keys, a field in a
			// panel keeps whatever it does with them); the on-page editor is a field that is the book's.
			if ((id === 'edit.find' || id === 'edit.replace') && isEditable(event.target)) {
				if (!isPageEditor(event.target)) return;
			}
			event.preventDefault();
			void run(id);
		};
		window.addEventListener('keydown', onKeyDown);
		return () => window.removeEventListener('keydown', onKeyDown);
	}, [run]);
}
