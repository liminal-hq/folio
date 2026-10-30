// Where the update check has got to, kept up to date as the app reports it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import type { UpdateState } from '@liminal-hq/folio-protocol/generated/UpdateState';
import { tauriUpdateClient, type UpdateClient } from '../services/updateClient';

/** The state of the update check; `null` until the app has said. */
export function useUpdateState(client: UpdateClient = tauriUpdateClient): UpdateState | null {
	const [state, setState] = useState<UpdateState | null>(null);
	useEffect(() => {
		let live = true;
		let stop: (() => void) | undefined;
		// An event is newer than an answer to `state()` asked before it, whenever that arrives.
		let heard = false;
		client
			.onState((next) => {
				heard = true;
				if (live) setState(next);
			})
			.then((off) => (live ? (stop = off) : off()))
			.catch(() => undefined);
		client
			.state()
			.then((found) => live && !heard && setState(found))
			.catch(() => undefined);
		return () => {
			live = false;
			stop?.();
		};
	}, [client]);
	return state;
}
