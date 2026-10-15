// The background each page of a spread shows: its own, else the spread's, else its master's
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Master } from '@liminal-hq/folio-protocol/generated/Master';
import type { ResolvedBackground } from '@liminal-hq/folio-protocol/generated/ResolvedBackground';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';

/**
 * The twin of Rust's `folio_model::resolve`, for drawing the canvas and the shelf's thumbnails
 * (which never fetch a layout). Both are tested against `fixtures/backgroundCases.json`. A page
 * with none of the three shows the paper, which is `null`.
 */
export function resolveBackgrounds(
	masters: readonly Master[],
	spread: Spread,
): (ResolvedBackground | null)[] {
	return spread.pages.map((page) => {
		if (page.bg) return { source: 'page', bg: page.bg };
		// A span needs two pages: a hand-edited book may carry one on a single page.
		if (spread.bgSpan && spread.pages.length > 1) return { source: 'span', bg: spread.bgSpan };
		const master = masters.find((m) => m.id === page.master);
		return master?.bg ? { source: 'master', bg: master.bg } : null;
	});
}
