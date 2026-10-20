// The keys for the book commands: save, open, new, close, undo and redo
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
		default:
			return null;
	}
}

/** Whether typing here belongs to the field, so undo and redo are its own. */
function isEditable(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	return target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
}

/** Runs book commands from the keyboard. A command that is not available does nothing. */
export function useBookKeys(run: (id: string) => Promise<void>): void {
	useEffect(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			const id = commandForKey(event);
			if (id === null) return;
			// The field handles its own undo and redo.
			if ((id === 'edit.undo' || id === 'edit.redo') && isEditable(event.target)) return;
			event.preventDefault();
			void run(id);
		};
		window.addEventListener('keydown', onKeyDown);
		return () => window.removeEventListener('keydown', onKeyDown);
	}, [run]);
}
