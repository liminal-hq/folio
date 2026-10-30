// The main window's screen: the welcome screen, or a book with its toolbar, stage, shelf and status
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { MenuBar } from '@liminal-hq/folio-chrome/MenuBar/MenuBar';
import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import { useEffect, useState } from 'react';
import { formatTrim, pageCount } from '../books/format';
import { useActiveBook, useBooks, useBooksSelector } from '../books/useBooks';
import { APP_MENU_MNEMONICS } from '../commands/appMenuModel';
import { requestJaxToggle } from '../jax/jaxToggle';
import { t } from '../i18n/messages';
import { AgentChip } from './AgentChip';
import { AGENT_EVENT } from './agentRequest';
import { ConnectAgentDialog } from './ConnectAgentDialog';
import { tauriAgentClient, type AgentClient } from '../services/agentClient';
import { UPDATE_EVENT } from './updateRequest';
import { UpdateDialog } from './UpdateDialog';
import { tauriUpdateClient, type UpdateClient } from '../services/updateClient';
import { tauriAppInfoClient, type AppInfoClient } from '../services/appInfoClient';
import { tauriCloseRequests, type CloseRequests } from '../services/closeRequests';
import { useSettings } from '../settings/useSettings';
import { WelcomeScreen } from '../welcome/WelcomeScreen';
import { AppMenu } from './AppMenu';
import { BookView } from './BookView';
import { AppTitleBar } from './AppTitleBar';
import { useCloseQuestion } from './CloseBookDialog';
import { useRecoveryQuestion } from './RecoveredBookDialog';
import styles from './MainScreen.module.css';
import { useAppMenu } from './useAppMenu';
import { useAutosave } from './useAutosave';
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

export function MainScreen({
	client = tauriAppInfoClient,
	closeRequests = tauriCloseRequests,
	agentClient = tauriAgentClient,
	updateClient = tauriUpdateClient,
}: {
	client?: AppInfoClient;
	closeRequests?: CloseRequests;
	agentClient?: AgentClient;
	updateClient?: UpdateClient;
}) {
	const settings = useSettings();
	const [agentsOpen, setAgentsOpen] = useState(false);
	useEffect(() => {
		const open = () => setAgentsOpen(true);
		window.addEventListener(AGENT_EVENT, open);
		return () => window.removeEventListener(AGENT_EVENT, open);
	}, []);
	const { ask, dialog } = useCloseQuestion();
	const { actions } = useBooks();
	const [updates, setUpdates] = useState<{ open: boolean; check: boolean }>({
		open: false,
		check: false,
	});
	useEffect(() => {
		const open = (event: Event) =>
			setUpdates({
				open: true,
				check: (event as CustomEvent<{ check?: boolean }>).detail?.check === true,
			});
		window.addEventListener(UPDATE_EVENT, open);
		return () => window.removeEventListener(UPDATE_EVENT, open);
	}, []);
	// Closing the window (the ✕, Alt+F4 or Quit) asks about every book with changes first.
	useEffect(() => {
		let stop: (() => void) | undefined;
		let live = true;
		void closeRequests
			.onCloseRequested(async () => (await actions.closeAll(ask)) === 'done')
			.then((off) => (live ? (stop = off) : off()))
			.catch(() => undefined);
		return () => {
			live = false;
			stop?.();
		};
	}, [closeRequests, actions, ask]);
	const recovery = useRecoveryQuestion();
	const menu = useAppMenu(ask, recovery.ask);
	useAutosave();
	useBookKeys(menu.run);
	const version = useAppVersion(client);
	const active = useActiveBook();
	const kinds = useBooksSelector((s) => s.kinds);
	const menuBar = settings.interface.menuBar;
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
						<BookView
							key={active.summary.id}
							id={active.summary.id}
							copy={active.copy}
							kindLabel={kindLabel}
							available={menu.available}
							run={menu.run}
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
						<AgentChip />
						{active && <span>{savedText(active.summary.path, active.summary.dirty)}</span>}
						{settings.jax !== 'off' &&
							(active ? (
								<button type="button" className={styles.perch} onClick={requestJaxToggle}>
									{t('status.jax')}
								</button>
							) : (
								<span>{t('status.jax')}</span>
							))}
						{version !== null && <span>{`${t('app.name')} ${version}`}</span>}
					</span>
				</footer>
			</div>
			{dialog}
			{recovery.dialog}
			<UpdateDialog
				open={updates.open}
				onClose={() => setUpdates((u) => ({ ...u, open: false }))}
				client={updateClient}
				checkOnOpen={updates.check}
				beforeInstall={async () => (await actions.closeAll(ask)) === 'done'}
			/>
			<ConnectAgentDialog
				open={agentsOpen}
				onClose={() => setAgentsOpen(false)}
				client={agentClient}
			/>
		</WindowFrame>
	);
}
