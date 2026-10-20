// The Check book dialog: everything preflight finds, worst first, each with a way to go and look
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { PreflightItem } from '@liminal-hq/folio-protocol/generated/PreflightItem';
import type { Severity } from '@liminal-hq/folio-protocol/generated/Severity';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';
import { t, type MessageKey } from '../i18n/messages';
import styles from './CheckBookDialog.module.css';
import { describe, rowsOf } from './preflightMessages';

interface CheckBookDialogProps {
	open: boolean;
	onClose(): void;
	client: BooksClient;
	id: string;
	book: Book;
	/** How each spread is named on the shelf, in order (`Cover`, `Pages 1–2`). */
	labels: string[];
	/** Goes to the spread and, when the issue is about a frame, selects it. */
	onShow(spread: number, frame: string | null): void;
	/** Sets the issues to leave alone, each by its key. */
	onIgnore(keys: string[]): void;
}

const GROUPS: ReadonlyArray<{ severity: Severity; heading: MessageKey }> = [
	{ severity: 'error', heading: 'preflight.group.error' },
	{ severity: 'warning', heading: 'preflight.group.warning' },
	{ severity: 'note', heading: 'preflight.group.note' },
];

export function CheckBookDialog({
	open,
	onClose,
	client,
	id,
	book,
	labels,
	onShow,
	onIgnore,
}: CheckBookDialogProps) {
	const [items, setItems] = useState<PreflightItem[] | null>(null);
	const [failed, setFailed] = useState(false);
	const [round, setRound] = useState(0);
	useEffect(() => {
		if (!open) return;
		let live = true;
		setItems(null);
		setFailed(false);
		client
			.preflightBook(id)
			.then((found) => live && setItems(found))
			.catch(() => live && setFailed(true));
		return () => {
			live = false;
		};
	}, [open, client, id, round]);

	const left = book.preflightIgnore ?? [];
	const ignore = (row: { items: PreflightItem[] }) => {
		onIgnore([
			...new Set([...left, ...row.items.map((i) => i.key).filter((k): k is string => !!k)]),
		]);
		setRound((n) => n + 1);
	};
	const restore = (row: { items: PreflightItem[] }) => {
		const gone = new Set(row.items.map((i) => i.key));
		onIgnore(left.filter((k) => !gone.has(k)));
		setRound((n) => n + 1);
	};
	const active = items?.filter((i) => !i.ignored) ?? null;
	const quiet = items?.filter((i) => i.ignored) ?? [];

	const where = (item: PreflightItem) => {
		const at = book.spreads.findIndex((s) => s.id === item.spread);
		return { at, name: labels[at] ?? '' };
	};

	return (
		<Dialog
			open={open}
			title={t('preflight.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton onClick={() => setRound((n) => n + 1)}>{t('preflight.again')}</DialogButton>
					<DialogButton variant="primary" onClick={onClose}>
						{t('preflight.done')}
					</DialogButton>
				</DialogActions>
			}
		>
			{failed && (
				<p role="alert" className={styles.note}>
					{t('preflight.failed')}
				</p>
			)}
			{!failed && items === null && (
				<p role="status" className={styles.note}>
					{t('preflight.checking')}
				</p>
			)}
			{active !== null && active.length === 0 && (
				<p role="status" className={styles.ok}>
					{t('preflight.clear')}
				</p>
			)}
			{active !== null &&
				GROUPS.map(({ severity, heading }) => {
					const rows = rowsOf(active.filter((item) => item.severity === severity));
					if (rows.length === 0) return null;
					return (
						<section key={severity} aria-labelledby={`preflight-${severity}`}>
							<h3 id={`preflight-${severity}`} className={styles.heading}>
								{t(heading)}{' '}
								<span className={styles.count}>
									{rows.reduce((total, row) => total + row.count, 0)}
								</span>
							</h3>
							<ul className={styles.list}>
								{rows.map((row, index) => {
									const { item, count } = row;
									const place = where(item);
									const words = describe(item, place.name, count);
									return (
										<li key={index} className={styles.row} data-severity={severity}>
											<span className={styles.dot} aria-hidden="true" />
											<span className={styles.text}>
												<span className={styles.problem}>{words.problem}</span>
												<span className={styles.fix}>{words.fix}</span>
											</span>
											<button type="button" className={styles.show} onClick={() => ignore(row)}>
												{t('preflight.ignore')}
											</button>
											{place.at >= 0 && (
												<button
													type="button"
													className={styles.show}
													onClick={() => {
														onShow(place.at, item.frame ?? null);
														onClose();
													}}
												>
													{t('preflight.show')}
												</button>
											)}
										</li>
									);
								})}
							</ul>
						</section>
					);
				})}
			{quiet.length > 0 && (
				<section aria-labelledby="preflight-ignored">
					<h3 id="preflight-ignored" className={styles.heading}>
						{t('preflight.group.ignored')} <span className={styles.count}>{quiet.length}</span>
					</h3>
					<ul className={styles.list}>
						{rowsOf(quiet).map((row, index) => {
							const place = where(row.item);
							const words = describe(row.item, place.name, row.count);
							return (
								<li key={index} className={styles.row} data-severity="note">
									<span className={styles.dot} aria-hidden="true" />
									<span className={styles.text}>
										<span className={styles.problem}>{words.problem}</span>
									</span>
									<button type="button" className={styles.show} onClick={() => restore(row)}>
										{t('preflight.restore')}
									</button>
								</li>
							);
						})}
					</ul>
				</section>
			)}
		</Dialog>
	);
}
