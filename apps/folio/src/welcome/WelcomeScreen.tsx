// The welcome screen: what to make, how to work, and where to pick up
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { KindInfo } from '@liminal-hq/folio-plugin-books';
import { useState, type KeyboardEvent } from 'react';
import { Segmented } from '../app/Segmented';
import { useWindows } from '../app/WindowsContext';
import { useBooks, useBooksSelector } from '../books/useBooks';
import { t } from '../i18n/messages';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import { useRecoveryQuestion } from '../app/RecoveredBookDialog';
import { greetingKey } from './greeting';
import { KindCard } from './KindCard';
import { recentEntry } from './recent';
import { RecoverySection } from './RecoverySection';
import styles from './WelcomeScreen.module.css';

/** Moves the choice with the arrow keys, as in a native radio group. */
function moveKind(kinds: KindInfo[], current: number, event: KeyboardEvent): number | null {
	const forward = event.key === 'ArrowRight' || event.key === 'ArrowDown';
	const back = event.key === 'ArrowLeft' || event.key === 'ArrowUp';
	if (!forward && !back) return null;
	return (current + (forward ? 1 : -1) + kinds.length) % kinds.length;
}

export function WelcomeScreen({ now = new Date() }: { now?: Date }) {
	const { store, actions } = useBooks();
	const kinds = useBooksSelector((s) => s.kinds);
	const error = useBooksSelector((s) => s.error);
	const settings = useSettings();
	const update = useUpdateSettings();
	const recovery = useRecoveryQuestion();
	const windows = useWindows();

	const [chosen, setChosen] = useState(0);
	const [trim, setTrim] = useState('0');
	const [mode, setMode] = useState(settings.mode);
	const [jax, setJax] = useState(settings.jax);
	const [starting, setStarting] = useState(false);

	const kind = kinds[chosen];

	const choose = (index: number) => {
		setChosen(index);
		setTrim('0');
	};

	const start = async () => {
		if (!kind) return;
		setStarting(true);
		await update({ mode, jax });
		await store.newBook({ kind: kind.kind, trimIndex: Number(trim) });
		setStarting(false);
	};

	return (
		<>
			<div className={styles.screen}>
				<div className={styles.main}>
					<div>
						<p className={styles.greeting}>{t(greetingKey(now))}</p>
						<h1 className={styles.heading}>{t('welcome.making')}</h1>
					</div>

					<div
						className={styles.kinds}
						role="radiogroup"
						aria-label={t('welcome.kinds')}
						onKeyDown={(event) => {
							const next = moveKind(kinds, chosen, event);
							if (next === null) return;
							event.preventDefault();
							choose(next);
							event.currentTarget.querySelectorAll<HTMLElement>('[role="radio"]')[next]?.focus();
						}}
					>
						{kinds.map((info, index) => (
							<KindCard
								key={info.kind}
								info={info}
								selected={index === chosen}
								onSelect={() => choose(index)}
							/>
						))}
					</div>

					{kind && (
						<div className={styles.setup}>
							<div className={styles.question}>
								<span className={styles.label}>{t('welcome.size')}</span>
								<Segmented
									label={t('welcome.size')}
									value={trim}
									options={kind.trims.map((choice, index) => ({
										value: String(index),
										label: choice.label,
									}))}
									onChange={setTrim}
								/>
							</div>
							<div className={styles.question}>
								<span className={styles.label}>{t('mode.label')}</span>
								<Segmented
									label={t('mode.label')}
									value={mode}
									options={[
										{ value: 'easy', label: t('mode.easy') },
										{ value: 'expert', label: t('mode.expert') },
									]}
									onChange={setMode}
								/>
							</div>
							<div className={styles.question}>
								<span className={styles.label}>{t('welcome.jax')}</span>
								<Segmented
									label={t('welcome.jax')}
									value={jax}
									options={[
										{ value: 'guide', label: t('preferences.jax.guide') },
										{ value: 'tips', label: t('preferences.jax.tips') },
										{ value: 'off', label: t('preferences.jax.off') },
									]}
									onChange={setJax}
								/>
							</div>
							<div className={styles.doors}>
								<button type="button" className={styles.start} disabled={starting} onClick={start}>
									{starting ? t('welcome.starting') : t('welcome.start', { kind: kind.label })}
								</button>
								<button
									type="button"
									className={styles.guide}
									onClick={() => void windows.open('WizardNew')}
								>
									{t('welcome.guide')}
								</button>
							</div>
						</div>
					)}

					{error && (
						<p role="alert" className={styles.problem}>
							{t('welcome.error', { message: error.message })}
						</p>
					)}
				</div>

				<aside className={styles.side}>
					<RecoverySection />
					<section aria-labelledby="welcome-recent">
						<h2 id="welcome-recent" className={styles.sideHeading}>
							{t('welcome.recent')}
						</h2>
						{settings.recentBooks.length === 0 ? (
							<p className={styles.muted}>{t('welcome.recent.empty')}</p>
						) : (
							<ul className={styles.list}>
								{settings.recentBooks.map((path) => {
									const entry = recentEntry(path);
									return (
										<li key={path}>
											<button
												type="button"
												className={styles.row}
												title={path}
												onClick={() => void actions.open(path, recovery.ask)}
											>
												<span className={styles.rowName}>{entry.name}</span>
												<span className={styles.rowFolder}>{entry.folder}</span>
											</button>
										</li>
									);
								})}
							</ul>
						)}
					</section>

					<section aria-labelledby="welcome-bring">
						<h2 id="welcome-bring" className={styles.sideHeading}>
							{t('welcome.bring')}
						</h2>
						<ul className={styles.list}>
							<li>
								<button
									type="button"
									className={styles.action}
									onClick={() => void actions.open(undefined, recovery.ask)}
								>
									{t('welcome.open')}
								</button>
							</li>
							{(['welcome.importWords', 'welcome.importLayout'] as const).map((key) => (
								<li key={key}>
									<button type="button" className={styles.action} disabled>
										<span>{t(key)}</span>
										<span className={styles.reason}>{t('menu.unavailable')}</span>
									</button>
								</li>
							))}
						</ul>
					</section>
				</aside>
			</div>
			{recovery.dialog}
		</>
	);
}
