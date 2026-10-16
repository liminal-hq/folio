// The Proofing group: every misspelt word in the book, with ways to deal with each
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { BookSpelling } from '@liminal-hq/folio-protocol/generated/BookSpelling';
import type { MisspeltWord } from '@liminal-hq/folio-protocol/generated/MisspeltWord';
import { useState } from 'react';
import { t } from '../i18n/messages';
import styles from './PagesPanel.module.css';

export interface ProofingPanelProps {
	/** What checking found, or `null` until it has been read. */
	found: BookSpelling | null;
	/** A name for each spread, as the shelf has them, in order, with each spread's id. */
	spreads: { id: string; label: string }[];
	/** Shows the word where it is on the page. */
	onShow(word: MisspeltWord): void;
	onIgnore(word: MisspeltWord): void;
	onAdd(word: MisspeltWord): void;
	/** Corrections for a word, best first. */
	onSuggest(word: string): Promise<string[]>;
	/** Puts a correction in place of the word. */
	onChange(word: MisspeltWord, to: string): void;
	/** Downloads the dictionary on offer. */
	onDownload?(id: string): void;
	/** The words this book leaves alone, so they can be taken off the list. */
	ignored?: string[];
	/** Checks a word again from now on, in this book. */
	onRestore?(word: string): void;
}

/** The words the book leaves alone, each with a way to stop. Nothing is drawn when there are none. */
function IgnoredWords({
	words,
	onRestore,
}: {
	words: string[];
	onRestore?: (word: string) => void;
}) {
	if (words.length === 0 || !onRestore) return null;
	return (
		<section className={styles.section} aria-label={t('proofing.ignored')}>
			<h2 className={styles.heading}>{t('proofing.ignored')}</h2>
			<ul className={styles.list}>
				{words.map((word) => (
					<li key={word} className={styles.misspelt}>
						<span className={styles.label}>
							<strong>{word}</strong>
						</span>
						<span className={styles.actions}>
							<button
								type="button"
								className={styles.action}
								aria-label={t('proofing.restore.for', { word })}
								onClick={() => onRestore(word)}
							>
								{t('proofing.restore')}
							</button>
						</span>
					</li>
				))}
			</ul>
		</section>
	);
}

const key = (w: MisspeltWord) => `${w.frame}/${w.row ?? ''}.${w.col ?? ''}/${w.start}`;

/**
 * The misspelt words, grouped by the spread they are on. A word can be shown on the page, ignored
 * in this book, added to the dictionary for every book, or changed to a suggestion (when its words
 * are plain text of a frame of its own).
 */
export function ProofingPanel({
	found,
	spreads,
	onShow,
	onIgnore,
	onAdd,
	onSuggest,
	onChange,
	onDownload,
	ignored = [],
	onRestore,
}: ProofingPanelProps) {
	const [open, setOpen] = useState<{ key: string; suggestions: string[] } | null>(null);

	if (found === null) return <p className={styles.note}>{t('proofing.checking')}</p>;
	if (!found.supported) {
		return (
			<section className={styles.section}>
				<p className={styles.note}>{t('proofing.unsupported')}</p>
				{found.offer && onDownload && (
					<button
						type="button"
						className={styles.action}
						onClick={() => onDownload(found.offer!.id)}
					>
						{t('proofing.download', { name: found.offer.name })}
					</button>
				)}
				<IgnoredWords words={ignored} onRestore={onRestore} />
			</section>
		);
	}
	const label = (id: string) => spreads.find((s) => s.id === id)?.label ?? '';
	const bySpread = spreads
		.map((s) => ({ ...s, words: found.words.filter((w) => w.spread === s.id) }))
		.filter((s) => s.words.length > 0);

	return (
		<>
			<p className={styles.note} role="status">
				{found.words.length === 0
					? t('proofing.none')
					: t('proofing.count', { count: found.words.length })}
				{found.truncated ? ` ${t('proofing.truncated')}` : ''}
			</p>
			{found.offer && onDownload && (
				<button type="button" className={styles.action} onClick={() => onDownload(found.offer!.id)}>
					{t('proofing.better', { name: found.offer.name })}
				</button>
			)}
			{bySpread.map((s) => (
				<section key={s.id} className={styles.section} aria-label={s.label}>
					<h2 className={styles.heading}>{s.label}</h2>
					<ul className={styles.list}>
						{s.words.map((w) => {
							const k = key(w);
							return (
								<li key={k} className={styles.misspelt}>
									<button
										type="button"
										className={styles.row}
										aria-label={t('proofing.show', { word: w.word, where: label(w.spread) })}
										onClick={() => onShow(w)}
									>
										<span className={styles.label}>
											<strong>{w.word}</strong>
											<span className={styles.is}>{w.context}</span>
										</span>
									</button>
									<span className={styles.actions}>
										<button
											type="button"
											className={styles.action}
											aria-label={t('proofing.ignore.for', { word: w.word })}
											onClick={() => onIgnore(w)}
										>
											{t('proofing.ignore')}
										</button>
										<button
											type="button"
											className={styles.action}
											aria-label={t('proofing.add.for', { word: w.word })}
											onClick={() => onAdd(w)}
										>
											{t('proofing.add')}
										</button>
										<button
											type="button"
											className={styles.action}
											aria-expanded={open?.key === k}
											aria-label={t('proofing.suggest.for', { word: w.word })}
											onClick={() =>
												open?.key === k
													? setOpen(null)
													: void onSuggest(w.word).then((suggestions) =>
															setOpen({ key: k, suggestions }),
														)
											}
										>
											{t('proofing.suggest')}
										</button>
									</span>
									{open?.key === k && (
										<ul
											className={styles.suggestions}
											aria-label={t('proofing.suggestions', { word: w.word })}
										>
											{!w.replaceable ? (
												<li className={styles.note}>{t('proofing.onThePage')}</li>
											) : open.suggestions.length === 0 ? (
												<li className={styles.note}>{t('spell.noSuggestions')}</li>
											) : (
												open.suggestions.map((to) => (
													<li key={to}>
														<button
															type="button"
															className={styles.action}
															onClick={() => {
																setOpen(null);
																onChange(w, to);
															}}
														>
															{to}
														</button>
													</li>
												))
											)}
										</ul>
									)}
								</li>
							);
						})}
					</ul>
				</section>
			))}
			<IgnoredWords words={ignored} onRestore={onRestore} />
		</>
	);
}
