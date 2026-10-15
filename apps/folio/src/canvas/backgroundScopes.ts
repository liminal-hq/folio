// Where a background can go on a spread, and what is there now
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Background } from '@liminal-hq/folio-protocol/generated/Background';
import type { BackgroundTarget } from '@liminal-hq/folio-protocol/generated/BackgroundTarget';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { MessageKey } from '../i18n/messages';

export interface BackgroundScope {
	id: string;
	label: MessageKey;
	target: BackgroundTarget;
}

/**
 * The places a background can go on `spread`: each page (the left and right of a pair, or the one
 * page), both pages through the gutter when there are two, and every page of the book. The pages
 * come first, in order, so the one at index `i` is at `i` for a pair. The cover offers its own page
 * only.
 */
export function backgroundTargets(spread: Spread): BackgroundScope[] {
	const pair = spread.pages.length > 1;
	const pages: BackgroundScope[] = spread.pages.map((page, i) => ({
		id: `page:${page.id}`,
		label: !pair
			? 'background.scope.page'
			: i === 0
				? 'background.scope.left'
				: 'background.scope.right',
		target: { type: 'page', spread: spread.id, page: page.id },
	}));
	const span: BackgroundScope[] = pair
		? [{ id: 'span', label: 'background.scope.span', target: { type: 'span', spread: spread.id } }]
		: [];
	const every: BackgroundScope = {
		id: 'every',
		label: 'background.scope.every',
		target: { type: 'everyPage' },
	};
	// The cover has no master, so "every page" would leave out the one page in view.
	const everywhere = spread.kind === 'cover' ? [] : [every];
	// The span is offered first on a pair, since a picture book usually paints the spread.
	return [...span, ...pages, ...everywhere];
}

/** What is set on `scope` now, or `null`. "Every page" shows the first master's background. */
export function currentBackground(
	book: Book,
	spread: Spread,
	scope: BackgroundScope,
): Background | null {
	const { target } = scope;
	if (target.type === 'span') return spread.bgSpan ?? null;
	if (target.type === 'page') {
		return spread.pages.find((p) => p.id === target.page)?.bg ?? null;
	}
	return (book.masterPages ?? []).find((m) => m.bg)?.bg ?? null;
}
