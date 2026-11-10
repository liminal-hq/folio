// The find and replace bar over the page: a query, next and previous, a count and, for Expert, options
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useId, useRef, useState, type KeyboardEvent } from 'react';
import { t } from '../i18n/messages';
import type { FindApi, FindOptions } from './useFind';
import styles from './FindBar.module.css';

export interface FindBarProps {
	find: FindApi;
	/** Expert shows the options row; Easy is a find box, next and previous, and replace. */
	expert: boolean;
}

/** The "3 of 12" the bar announces. */
export function countText(
	find: Pick<FindApi, 'query' | 'matches' | 'index' | 'truncated' | 'error'>,
) {
	if (find.error || find.query === '') return '';
	const total = find.matches.length;
	if (total === 0) return t('find.count.none');
	const n = (find.index ?? 0) + 1;
	return find.truncated ? t('find.count.more', { n, total }) : t('find.count', { n, total });
}

/**
 * Finds words in the book and, when asked, replaces them. Enter goes to the next match and
 * Shift+Enter to the previous one; Escape closes the bar. The bar is a region of its own, and the
 * count is a polite live region so a screen reader hears "3 of 12" as the match changes.
 */
export function FindBar({ find, expert }: FindBarProps) {
	const field = useRef<HTMLInputElement | null>(null);
	const [more, setMore] = useState(false);
	const optionsId = useId();
	const count = countText(find);

	// Opening the bar, or asking for it again, puts the cursor in the query with its words selected.
	useEffect(() => {
		field.current?.focus();
		field.current?.select();
	}, [find.open, find.asked]);

	const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
		if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			find.close();
			return;
		}
		const mod = event.ctrlKey || event.metaKey;
		if (mod && !event.altKey && !event.shiftKey && ['f', 'h'].includes(event.key.toLowerCase())) {
			// The bar's own keys: the window's handler leaves a field alone.
			event.preventDefault();
			find.show(event.key.toLowerCase() === 'h');
			field.current?.focus();
			field.current?.select();
		}
	};

	const toggle = (key: keyof FindOptions) =>
		find.setOptions({ ...find.options, [key]: !find.options[key] });

	return (
		<section
			className={styles.bar}
			data-find-bar=""
			role="search"
			aria-label={find.replacing ? t('find.replace.label') : t('find.label')}
			onKeyDown={onKeyDown}
		>
			<div className={styles.row}>
				<input
					ref={field}
					className={styles.field}
					type="text"
					value={find.query}
					aria-label={t('find.query')}
					aria-invalid={find.error ? true : undefined}
					placeholder={t('find.query')}
					spellCheck={false}
					autoComplete="off"
					onChange={(event) => find.setQuery(event.target.value)}
					onKeyDown={(event) => {
						if (event.key === 'Enter') {
							event.preventDefault();
							find.step(event.shiftKey);
						}
					}}
				/>
				<span className={styles.count} role="status" aria-live="polite">
					{find.error ?? count}
				</span>
				<button
					type="button"
					className={styles.button}
					aria-label={t('find.previous')}
					title={t('find.previous')}
					disabled={find.matches.length === 0}
					onClick={() => find.step(true)}
				>
					<span aria-hidden="true">↑</span>
				</button>
				<button
					type="button"
					className={styles.button}
					aria-label={t('find.next')}
					title={t('find.next')}
					disabled={find.matches.length === 0}
					onClick={() => find.step(false)}
				>
					<span aria-hidden="true">↓</span>
				</button>
				{expert && (
					<button
						type="button"
						className={styles.text}
						aria-expanded={more}
						aria-controls={optionsId}
						onClick={() => setMore((was) => !was)}
					>
						{t('find.options')}
					</button>
				)}
				<button
					type="button"
					className={styles.button}
					aria-label={t('find.close')}
					title={t('find.close')}
					onClick={find.close}
				>
					<span aria-hidden="true">×</span>
				</button>
			</div>
			{expert && more && (
				<div className={styles.row} id={optionsId}>
					{(
						[
							['matchCase', 'find.matchCase'],
							['wholeWord', 'find.wholeWord'],
							['regex', 'find.regex'],
						] as const
					).map(([key, label]) => (
						<label key={key} className={styles.option}>
							<input type="checkbox" checked={find.options[key]} onChange={() => toggle(key)} />
							{t(label)}
						</label>
					))}
				</div>
			)}
			{find.replacing && (
				<div className={styles.row}>
					<input
						className={styles.field}
						type="text"
						value={find.replacement}
						aria-label={t('find.replacement')}
						placeholder={t('find.replacement')}
						spellCheck={false}
						autoComplete="off"
						onChange={(event) => find.setReplacement(event.target.value)}
						onKeyDown={(event) => {
							if (event.key === 'Enter') {
								event.preventDefault();
								void find.replaceOne();
							}
						}}
					/>
					<button
						type="button"
						className={styles.text}
						disabled={find.index === null}
						onClick={() => void find.replaceOne()}
					>
						{t('find.replaceOne')}
					</button>
					<button
						type="button"
						className={styles.text}
						disabled={find.matches.length === 0}
						onClick={() => void find.replaceAll()}
					>
						{t('find.replaceAll')}
					</button>
				</div>
			)}
			{find.notice && (
				<p className={styles.notice} role="status">
					{find.notice}
				</p>
			)}
		</section>
	);
}
