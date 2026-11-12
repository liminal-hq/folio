// What the page can ask of the book view about links: follow one, edit one, or take one away
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The things the page does with a link. The book view that is open sets them. */
export interface LinkActions {
	/** Opens the address in the system's program, or jumps to the spread it names. */
	follow(href: string): void;
	/** Opens the link dialog for the words the editor's caret or selection is in. */
	edit(): void;
	/** Takes the link off the words the editor's caret or selection is in. */
	remove(): void;
}

const none: LinkActions = {
	follow: () => undefined,
	edit: () => undefined,
	remove: () => undefined,
};
let current: LinkActions = none;

/** Sets what the page's links do; answers a function that puts back what was there. */
export function setLinkActions(actions: LinkActions): () => void {
	const before = current;
	current = actions;
	return () => {
		if (current === actions) current = before;
	};
}

export const linkActions = (): LinkActions => current;
