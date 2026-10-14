// The main window's screen: the welcome screen, or a book with its toolbar, stage, shelf and status
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { MenuBar } from '@liminal-hq/folio-chrome/MenuBar/MenuBar';
import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import type { CSSProperties } from 'react';
import { useEffect, useState } from 'react';
import { formatTrim, pageCount } from '../books/format';
import { shelfLabels } from '../books/shelfLabels';
import { frameIntents } from '../canvas/frameMenu';
import { Shelf } from '../canvas/Shelf';
import { Stage } from '../canvas/Stage';
import { useSelection } from '../canvas/useSelection';
import { useSpreadLayout } from '../canvas/useSpreadLayout';
import { useActiveBook, useBooks, useBooksSelector } from '../books/useBooks';
import { APP_MENU_MNEMONICS } from '../commands/appMenuModel';
import { t } from '../i18n/messages';
import { tauriAppInfoClient, type AppInfoClient } from '../services/appInfoClient';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import { WelcomeScreen } from '../welcome/WelcomeScreen';
import { AppMenu } from './AppMenu';
import { AppTitleBar } from './AppTitleBar';
import { useCloseQuestion } from './CloseBookDialog';
import styles from './MainScreen.module.css';
import { Segmented } from './Segmented';
import { useAppMenu } from './useAppMenu';
import { useBookKeys } from './useBookKeys';

/** The app's version, or `null` until it is known (and for good outside Tauri). */
function useAppVersion(client: AppInfoClient): string | null {
	const [version, setVersion] = useState<string | null>(null);
	useEffect(() => {
		let active = true;
		client
			.appInfo()
			.then((info) => {
				if (active) setVersion(info.version);
			})
			.catch(() => {
				// Not in Tauri (a plain browser, or a test).
			});
		return () => {
			active = false;
		};
	}, [client]);
	return version;
}

/** What the status bar says about where the book is kept. */
function savedText(path: string | null, dirty: boolean): string {
	if (path === null) return t('status.notSaved');
	if (dirty) return t('status.unsaved');
	return t('status.saved.to', { file: path.split(/[/\\]/).pop() ?? path });
}

export function MainScreen({ client = tauriAppInfoClient }: { client?: AppInfoClient }) {
	const settings = useSettings();
	const update = useUpdateSettings();
	const { ask, dialog } = useCloseQuestion();
	const menu = useAppMenu(ask);
	useBookKeys(menu.run);
	const version = useAppVersion(client);
	const active = useActiveBook();
	const kinds = useBooksSelector((s) => s.kinds);
	const menuBar = settings.interface.menuBar;
	const { client: booksClient, store: booksStore } = useBooks();
	const [spreadIndex, setSpreadIndex] = useState(0);
	const spreads = active?.copy.book.spreads ?? [];
	const shown = Math.max(0, Math.min(spreadIndex, spreads.length - 1));
	const spread = spreads[shown] ?? null;
	const labels = active ? shelfLabels(active.copy.book) : [];
	const layout = useSpreadLayout(
		booksClient,
		active?.summary.id ?? '',
		active && spread ? spread.id : null,
		active?.copy.revision ?? 0,
	);
	const [selection, select] = useSelection(
		spread?.id ?? null,
		spread?.frames.map((f) => f.id) ?? [],
	);
	const inspectorWidth = { '--folio-inspector-width': `${settings.interface.sidebarWidth}px` };

	const kindLabel = active
		? (kinds.find((k) => k.kind === active.summary.kind)?.label ?? active.summary.kind)
		: '';
	const title = active
		? t('window.main.book', { title: active.summary.title, kind: kindLabel })
		: t('window.main.title');

	return (
		<WindowFrame>
			<div className={styles.screen}>
				<AppTitleBar
					title={title}
					start={<AppMenu items={menu.items} onSelect={menu.onSelect} menuBar={menuBar} />}
				/>
				{menuBar && (
					<MenuBar
						items={menu.barItems}
						onSelect={menu.onSelect}
						label={t('menu.bar')}
						moreLabel={t('menu.more')}
						mnemonics={APP_MENU_MNEMONICS}
					/>
				)}
				{active === null ? (
					<WelcomeScreen />
				) : (
					<>
						<div className={styles.toolbar} role="toolbar" aria-label={t('region.toolbar')}>
							<p className={styles.toolbarNote}>
								{settings.interface.toolbar === 'ribbon'
									? t('region.toolbar.ribbon')
									: t('region.toolbar.classic')}
							</p>
							<div className={styles.mode}>
								<span>{t('mode.label')}</span>
								<Segmented
									label={t('mode.label')}
									value={settings.mode}
									options={[
										{ value: 'easy', label: t('mode.easy') },
										{ value: 'expert', label: t('mode.expert') },
									]}
									onChange={(mode) => void update({ mode })}
								/>
							</div>
						</div>
						<main className={styles.body} style={inspectorWidth as CSSProperties}>
							{spread && (
								<Stage
									book={active.copy.book}
									spread={spread}
									label={labels[shown] ?? ''}
									layout={layout}
									selection={selection}
									onSelect={select}
									onMove={(id, box) =>
										booksStore.apply({
											type: 'MoveFrame',
											spread: spread.id,
											frame: id,
											box: { x: box.x, y: box.y, w: box.w, h: box.h },
										})
									}
									onEditText={(id, text) =>
										void booksStore.apply({ type: 'SetText', frame: id, text })
									}
									onCommand={(command, ids) => {
										for (const id of ids) {
											const target = spread.frames.find((f) => f.id === id);
											if (!target) continue;
											for (const intent of frameIntents(command, spread.id, target)) {
												void booksStore.apply(intent);
											}
										}
									}}
								/>
							)}
							<aside className={styles.inspector} aria-label={t('region.inspector')}>
								<h2 className={styles.inspectorTitle}>{t('region.inspector')}</h2>
								<p className={styles.muted}>{t('region.inspector.empty')}</p>
							</aside>
						</main>
						<Shelf
							book={active.copy.book}
							labels={labels}
							selected={shown}
							onSelect={setSpreadIndex}
						/>
					</>
				)}
				<footer className={styles.status} aria-label={t('status.bar')}>
					<span>
						{active
							? t('status.book', {
									kind: kindLabel,
									trim: formatTrim(active.copy.book.trim, settings.units),
									pages: (() => {
										const pages = pageCount(active.copy.book);
										return pages === 1
											? t('status.pages.one')
											: t('status.pages.many', { count: pages });
									})(),
									layout: t(
										active.copy.book.layout === 'flowing' ? 'layout.flowing' : 'layout.fixed',
									),
								})
							: t('status.noBook')}
					</span>
					<span className={styles.statusGroup}>
						{active && <span>{savedText(active.summary.path, active.summary.dirty)}</span>}
						{settings.jax !== 'off' && <span>{t('status.jax')}</span>}
						{version !== null && <span>{`${t('app.name')} ${version}`}</span>}
					</span>
				</footer>
			</div>
			{dialog}
		</WindowFrame>
	);
}
