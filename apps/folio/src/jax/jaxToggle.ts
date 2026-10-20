// Asks the open book's window to show or hide Jax's panel, for the status bar's Jax label
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The event the book's window listens for. */
export const JAX_TOGGLE_EVENT = 'folio:jax-toggle';

/** Shows Jax's panel, or puts him away when it is showing. Nothing happens with no book open. */
export function requestJaxToggle(): void {
	window.dispatchEvent(new CustomEvent(JAX_TOGGLE_EVENT));
}
