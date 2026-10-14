// Starts the window's settings store and shares it with the page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useMemo, type ReactNode } from 'react';
import type { SettingsClient } from './settingsClient';
import { createSettingsStore } from './settingsStore';
import { SettingsContext } from './useSettings';

export function SettingsProvider({
	client,
	children,
}: {
	client: SettingsClient;
	children: ReactNode;
}) {
	const store = useMemo(() => createSettingsStore(client), [client]);

	useEffect(() => {
		let active = true;
		let stop: (() => void) | undefined;
		store
			.start()
			.then((off) => {
				if (active) stop = off;
				else off();
			})
			.catch(() => {
				// Not in Tauri: the defaults stay in force.
			});
		return () => {
			active = false;
			stop?.();
		};
	}, [store]);

	return <SettingsContext.Provider value={store}>{children}</SettingsContext.Provider>;
}
