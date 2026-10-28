// Counts the open dialogs, so the window frame can dim just its own surface
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// A native dialog's backdrop covers the whole viewport. In a frameless transparent window that
// includes the margin that holds the window's drop shadow, so dimming it shows the square window
// around the rounded one. A window frame dims its own rounded surface instead, from this count.

let open = 0;
const listeners = new Set<() => void>();

/** Notes that a dialog opened (`1`) or closed (`-1`). */
export function dialogOpened(change: 1 | -1): void {
	open = Math.max(0, open + change);
	for (const listener of listeners) listener();
}

/** Calls `listener` whenever a dialog opens or closes; returns the way to stop. */
export function subscribeDimming(listener: () => void): () => void {
	listeners.add(listener);
	return () => listeners.delete(listener);
}

/** Whether any dialog is open. */
export function isDimmed(): boolean {
	return open > 0;
}
