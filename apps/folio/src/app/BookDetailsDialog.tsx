// The Book details dialog: the title, author, description and ISBNs a cover and a store show
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Paper } from '@liminal-hq/folio-protocol/generated/Paper';
import type { Details } from '@liminal-hq/folio-protocol/generated/Details';
import { useEffect, useState } from 'react';
import { t } from '../i18n/messages';
import styles from './BookDetailsDialog.module.css';
import { isValidIsbn13 } from './isbn';

interface BookDetailsDialogProps {
	open: boolean;
	onClose(): void;
	book: Book;
	/** Sets the details, as one step of undo. */
	onSave(details: Details): void;
	/** Sets the paper, as its own undo step, when it was changed. */
	onPaper(paper: Paper): void;
}

type IsbnKind = 'paperback' | 'hardcover' | 'ebook';
const ISBNS: readonly IsbnKind[] = ['paperback', 'hardcover', 'ebook'];

/** What the form holds: the first author is the one the cover shows, and any others are kept. */
function formOf(book: Book) {
	return {
		title: book.title,
		subtitle: book.subtitle ?? '',
		author: book.authors[0]?.name ?? '',
		description: book.metadata.description,
		isbn: {
			paperback: book.metadata.isbn.paperback ?? '',
			hardcover: book.metadata.isbn.hardcover ?? '',
			ebook: book.metadata.isbn.ebook ?? '',
		},
	};
}

export function BookDetailsDialog({
	open,
	onClose,
	book,
	onSave,
	onPaper,
}: BookDetailsDialogProps) {
	const [form, setForm] = useState(() => formOf(book));
	const [paper, setPaper] = useState<Paper>(book.paper ?? 'white');
	useEffect(() => {
		if (open) {
			setForm(formOf(book));
			setPaper(book.paper ?? 'white');
		}
		// The form starts from the book each time the dialog opens, not as the book changes under it.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open]);

	const badIsbn = (kind: IsbnKind) =>
		form.isbn[kind].trim() !== '' && !isValidIsbn13(form.isbn[kind]);
	const valid = form.title.trim() !== '' && !ISBNS.some(badIsbn);

	const save = () => {
		if (paper !== (book.paper ?? 'white')) onPaper(paper);
		const [first, ...others] = book.authors;
		const name = form.author.trim();
		const authors = name ? [{ name, role: first?.role ?? ('author' as const) }, ...others] : others;
		onSave({
			title: form.title.trim(),
			subtitle: form.subtitle.trim() || null,
			authors,
			description: form.description.trim(),
			isbn: {
				paperback: form.isbn.paperback.trim() || null,
				hardcover: form.isbn.hardcover.trim() || null,
				ebook: form.isbn.ebook.trim() || null,
			},
		});
		onClose();
	};

	return (
		<Dialog
			open={open}
			title={t('details.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton onClick={onClose}>{t('details.cancel')}</DialogButton>
					<DialogButton variant="primary" disabled={!valid} onClick={save}>
						{t('details.save')}
					</DialogButton>
				</DialogActions>
			}
		>
			<div className={styles.form}>
				<label className={styles.field}>
					<span>{t('details.bookTitle')}</span>
					<input
						value={form.title}
						aria-invalid={form.title.trim() === ''}
						onChange={(e) => setForm({ ...form, title: e.target.value })}
					/>
				</label>
				<label className={styles.field}>
					<span>{t('details.subtitle')}</span>
					<input
						value={form.subtitle}
						onChange={(e) => setForm({ ...form, subtitle: e.target.value })}
					/>
				</label>
				<label className={styles.field}>
					<span>{t('details.author')}</span>
					<input
						value={form.author}
						onChange={(e) => setForm({ ...form, author: e.target.value })}
					/>
				</label>
				<label className={styles.field}>
					<span>{t('details.description')}</span>
					<textarea
						rows={4}
						value={form.description}
						onChange={(e) => setForm({ ...form, description: e.target.value })}
					/>
					<span className={styles.hint}>{t('details.descriptionHint')}</span>
				</label>
				<label className={styles.field}>
					<span>{t('details.paper')}</span>
					<select value={paper} onChange={(e) => setPaper(e.target.value as Paper)}>
						<option value="white">{t('details.paper.white')}</option>
						<option value="cream">{t('details.paper.cream')}</option>
						<option value="colour">{t('details.paper.colour')}</option>
					</select>
					<span className={styles.hint}>{t('details.paperHint')}</span>
				</label>
				{ISBNS.map((kind) => (
					<label key={kind} className={styles.field}>
						<span>{t(`details.isbn.${kind}`)}</span>
						<input
							value={form.isbn[kind]}
							aria-invalid={badIsbn(kind)}
							onChange={(e) => setForm({ ...form, isbn: { ...form.isbn, [kind]: e.target.value } })}
						/>
						{badIsbn(kind) && (
							<span role="alert" className={styles.problem}>
								{t('details.isbn.bad')}
							</span>
						)}
						{kind === 'paperback' && !badIsbn(kind) && (
							<span className={styles.hint}>{t('details.isbn.barcodeHint')}</span>
						)}
					</label>
				))}
			</div>
		</Dialog>
	);
}
