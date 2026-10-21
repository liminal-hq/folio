// The Book tools window: the utility window that hosts one workbench at a time
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import { useEffect, useState } from 'react';
import { pageCount } from '../books/format';
import { useActiveBook, useBooks } from '../books/useBooks';
import { isArea, type ToolArea } from '../booktools/tools';
import { ToolShell } from '../booktools/ToolShell';
import { t } from '../i18n/messages';
import { useSettings } from '../settings/useSettings';
import { tauriToolsClient, type ToolsClient } from '../services/windowsClient';
import { AppTitleBar } from './AppTitleBar';
import styles from './BookToolsScreen.module.css';

/**
 * The window follows the book the main window shows (its books store is made to), and a tool's
 * change goes through the same store, so it is one undo step of that book and the main window shows
 * it at once.
 */
export function BookToolsScreen({ client = tauriToolsClient }: { client?: ToolsClient }) {
	const { store, client: books } = useBooks();
	const active = useActiveBook();
	const settings = useSettings();
	const [area, setArea] = useState<ToolArea>('structure');

	useEffect(() => {
		let live = true;
		let stop: (() => void) | undefined;
		client
			.area()
			.then((found) => live && isArea(found) && setArea(found))
			.catch(() => {});
		client
			.onArea((next) => isArea(next) && setArea(next))
			.then((off) => (live ? (stop = off) : off()))
			.catch(() => {});
		return () => {
			live = false;
			stop?.();
		};
	}, [client]);

	const book = active?.copy.book;
	return (
		<WindowFrame>
			<div className={styles.screen}>
				<AppTitleBar title={`${t('window.bookTools.title')} · ${t(`tools.area.${area}`)}`} />
				<ToolShell
					area={area}
					onArea={setArea}
					about={
						active && book
							? t('tools.about', { title: active.summary.title, pages: pageCount(book) })
							: null
					}
					work={
						active && book
							? {
									client: books,
									id: active.summary.id,
									book,
									expert: settings.mode === 'expert',
									apply: (intent) => store.apply(intent),
								}
							: null
					}
				/>
			</div>
		</WindowFrame>
	);
}
