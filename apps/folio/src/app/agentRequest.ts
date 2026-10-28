// Asks the main window to show the Connect an agent dialog, for the menu command
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The event the main window listens for. */
export const AGENT_EVENT = 'folio:agents';

/** Opens the Connect an agent dialog. */
export function requestAgentDialog(): void {
	window.dispatchEvent(new CustomEvent(AGENT_EVENT));
}
