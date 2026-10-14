// Which frames are selected on the spread being shown
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useState } from 'react';

/**
 * The selected frame ids. The selection belongs to one spread: showing another empties it, and an id
 * whose frame has gone (deleted, undone away) drops out instead of lingering.
 */
export function useSelection(
	spreadId: string | null,
	frameIds: string[],
): [string[], (ids: string[]) => void] {
	const [held, setHeld] = useState<{ spread: string | null; ids: string[] }>({
		spread: null,
		ids: [],
	});
	const ids = held.spread === spreadId ? held.ids.filter((id) => frameIds.includes(id)) : [];
	return [ids, (next) => setHeld({ spread: spreadId, ids: next })];
}
