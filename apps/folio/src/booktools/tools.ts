// The Book tools window's areas and the tools in each, which a tool adds itself to
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { ComponentType } from 'react';
import type { BooksClient } from '../books/booksClient';
import type { MessageKey } from '../i18n/messages';

/** The groups of tools the window offers, in the order it lists them. */
export const AREAS = ['structure', 'typesetting', 'pictures', 'publishing'] as const;
export type ToolArea = (typeof AREAS)[number];

/** Areas the design plans and Folio has not built yet; the switcher lists them as coming. */
export const LATER_AREAS = ['proofing', 'series', 'accessibility', 'pictureBook'] as const;

export function isArea(value: unknown): value is ToolArea {
	return typeof value === 'string' && (AREAS as readonly string[]).includes(value);
}

/** What a tool is given: the book it works on, and how to look at and change it. */
export interface ToolProps {
	client: BooksClient;
	/** The id of the open book. */
	id: string;
	book: Book;
	/** Plain words in Easy, numbers too in Expert. */
	expert: boolean;
	/** Changes the book; every call is one undo step of the book's own history. */
	apply(intent: Intent): Promise<void>;
	/** Says what a change did, on the status line of the window. */
	say(message: string): void;
}

export interface ToolDef {
	/** Unique, such as `structure.copyright`. */
	id: string;
	area: ToolArea;
	titleKey: MessageKey;
	/** One line under the title saying what the tool is for. */
	descKey: MessageKey;
	Component: ComponentType<ToolProps>;
}

/** Every tool, in the order the sidebar lists them within an area. */
export const TOOLS: ToolDef[] = [];

export function toolsIn(area: ToolArea): ToolDef[] {
	return TOOLS.filter((tool) => tool.area === area);
}
