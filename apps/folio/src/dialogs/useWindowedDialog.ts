// Opens a dialog as a window when the setting asks for it, and says whether the page should show it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useRef } from 'react';
import type { DialogPayloads, DialogResults, OpenableKind } from './dialogKinds';
import { openDialog } from './openDialog';

/**
 * Keeps a dialog the page controls (`open`) and a dialog window in step. With `windowed` on, the
 * dialog opens as a window when `open` becomes true and `onEnded` runs with its result (null when
 * it was closed) so the page can clear `open`; the window is never shown twice. It returns whether
 * the page's own dialog should be shown.
 */
export function useWindowedDialog<K extends OpenableKind>(
	kind: K,
	open: boolean,
	windowed: boolean,
	payload: DialogPayloads[K],
	onEnded: (result: DialogResults[K] | null) => void,
): boolean {
	const latest = useRef({ payload, onEnded });
	latest.current = { payload, onEnded };
	const inFlight = useRef(false);
	useEffect(() => {
		if (!open || !windowed || inFlight.current) return;
		inFlight.current = true;
		openDialog(kind, latest.current.payload)
			.then((result) => latest.current.onEnded(result))
			// Another modal dialog is open already, or the window could not be made.
			.catch(() => latest.current.onEnded(null))
			.finally(() => {
				inFlight.current = false;
			});
	}, [kind, open, windowed]);
	return open && !windowed;
}
