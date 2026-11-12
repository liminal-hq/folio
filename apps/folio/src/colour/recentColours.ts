// The colours last used, kept as a per-user setting and offered first by every colour picker
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { parseColour } from '@liminal-hq/folio-chrome/ColourPicker/colourMaths';
import { useCallback, useMemo } from 'react';
import { useSettings, useUpdateSettings } from '../settings/useSettings';

/** How many recent colours are kept; the twin of `RECENT_COLOURS_MAX` in `folio-settings`. */
export const RECENT_COLOURS_MAX = 8;

/** `recent` with `colour` at the front, once, as `#rrggbb`; a colour that does not read changes nothing. */
export function pushRecentColour(recent: readonly string[], colour: string): string[] {
	const hex = parseColour(colour);
	if (hex === null) return [...recent];
	return [hex, ...recent.filter((c) => c !== hex)].slice(0, RECENT_COLOURS_MAX);
}

/** The recent colours and a way to record one, through the settings plugin. */
export function useRecentColours(): { recents: string[]; record(colour: string): void } {
	const recents = useSettings().recentColours;
	const update = useUpdateSettings();
	const record = useCallback(
		(colour: string) => {
			const next = pushRecentColour(recents, colour);
			if (next.length === recents.length && next.every((c, i) => c === recents[i])) return;
			void update({ recentColours: next });
		},
		[recents, update],
	);
	return useMemo(() => ({ recents, record }), [recents, record]);
}
