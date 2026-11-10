// The link dialog: set where some words link to, change it, or take the link away
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import { useEffect, useRef, useState } from 'react';
import { t } from '../i18n/messages';
import { checkHref, spreadOfHref } from '../text/links';
import styles from './LinkDialog.module.css';

export interface LinkDialogProps {
	open: boolean;
	onClose(): void;
	/** The words the link covers, shown so it is clear what is being linked. */
	words: string;
	/** The address now, or empty when the words have no link. */
	href: string;
	/** The words already link somewhere (or to several places), so this edits. */
	editing: boolean;
	/** The words link to different places. */
	mixed: boolean;
	/** Whether there are words to link: a caret that is not in a link has none. */
	hasWords: boolean;
	/** The name of each spread of the book, for the list of pages. */
	spreads: readonly string[];
	/** Sets the link; the address has been checked. */
	onSet(href: string): void;
	onRemove(): void;
}

export function LinkDialog({
	open,
	onClose,
	words,
	href,
	editing,
	mixed,
	hasWords,
	spreads,
	onSet,
	onRemove,
}: LinkDialogProps) {
	const [typed, setTyped] = useState('');
	const field = useRef<HTMLInputElement | null>(null);
	useEffect(() => {
		if (open) setTyped(href);
		// The form starts from the link each time the dialog opens.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open]);

	const check = checkHref(typed, spreads.length);
	const problem = check.ok || check.reason === 'empty' ? null : check.reason;
	const page = spreadOfHref(check.ok ? check.href : typed.trim());
	const set = () => {
		if (!check.ok) return;
		onSet(check.href);
		onClose();
	};

	return (
		<Dialog
			open={open}
			title={editing ? t('link.title.edit') : t('link.title')}
			size="medium"
			onClose={onClose}
			initialFocus={field}
			footer={
				<DialogActions>
					{editing && (
						<DialogButton
							variant="danger"
							onClick={() => {
								onRemove();
								onClose();
							}}
						>
							{t('link.remove')}
						</DialogButton>
					)}
					<DialogButton onClick={onClose}>{t('link.cancel')}</DialogButton>
					<DialogButton variant="primary" disabled={!hasWords || !check.ok} onClick={set}>
						{editing ? t('link.save') : t('link.set')}
					</DialogButton>
				</DialogActions>
			}
		>
			{hasWords ? (
				<form
					className={styles.form}
					onSubmit={(event) => {
						event.preventDefault();
						set();
					}}
				>
					<p className={styles.words}>{t('link.words', { words: `“${words}”` })}</p>
					<label className={styles.field}>
						<span>{t('link.address')}</span>
						<input
							ref={field}
							value={typed}
							type="text"
							inputMode="url"
							autoComplete="off"
							spellCheck={false}
							placeholder="example.org"
							aria-invalid={problem !== null}
							aria-describedby="link-hint"
							onChange={(event) => setTyped(event.target.value)}
						/>
					</label>
					{problem ? (
						<p id="link-hint" className={styles.problem} role="alert">
							{t(`link.problem.${problem}`)}
						</p>
					) : (
						<p id="link-hint" className={styles.hint}>
							{t('link.address.hint')}
						</p>
					)}
					{spreads.length > 0 && (
						<label className={styles.field}>
							<span>{t('link.page')}</span>
							<select
								value={page !== null && page <= spreads.length ? String(page) : ''}
								onChange={(event) =>
									setTyped(event.target.value ? `#spread-${event.target.value}` : '')
								}
							>
								<option value="">{t('link.page.none')}</option>
								{spreads.map((label, i) => (
									<option key={i} value={i + 1}>
										{t('link.page.option', { n: i + 1, label })}
									</option>
								))}
							</select>
						</label>
					)}
					{mixed && <p className={styles.hint}>{t('link.mixed')}</p>}
				</form>
			) : (
				<p className={styles.hint} role="status">
					{t('link.noWords')}
				</p>
			)}
		</Dialog>
	);
}
