// Opens the books the system asks for, as File → Open does, and says when one cannot be opened
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useRef } from 'react';
import { useBooks } from '../books/useBooks';
import { RECENT_PREFIX } from '../commands/appMenuModel';
import { t } from '../i18n/messages';
import type { OpenFilesClient } from '../services/openFilesClient';

/**
 * Opens each path the app is asked to open, one after another, through `run` (the menu's command
 * runner, so a book being shown is closed first, with its "save changes?" question, and a crash's
 * recovery copy is offered). A path that does not open is reported with the reason, since the
 * window may be showing another book and the start screen's own notice would not be seen.
 */
export function useOpenFiles(client: OpenFilesClient, run: (id: string) => Promise<void>): void {
	const { store } = useBooks();
	// The latest runner, so a changing book does not tear down the listener and miss a path.
	const latest = useRef(run);
	latest.current = run;

	useEffect(() => {
		let live = true;
		let draining = Promise.resolve();

		const drain = () => {
			// Taken one batch at a time, so two requests close together do not interleave their questions.
			draining = draining.then(async () => {
				const paths = await client.take().catch(() => [] as string[]);
				for (const path of paths) {
					if (!live) return;
					store.state.setState({ error: null });
					await latest.current(`${RECENT_PREFIX}${path}`);
					const { error } = store.state.getState();
					if (error !== null) {
						await client
							.report(t('openFile.failed', { path, message: error.message }))
							.catch(() => undefined);
					}
				}
			});
		};

		let stop: (() => void) | undefined;
		// Listening comes first, so a path that arrives while the page loads is taken either way.
		void client
			.onWaiting(drain)
			.then((off) => (live ? (stop = off) : off()))
			.catch(() => undefined)
			.then(drain);
		return () => {
			live = false;
			stop?.();
		};
	}, [client, store]);
}
