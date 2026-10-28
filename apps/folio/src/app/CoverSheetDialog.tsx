// The cover sheet preview: back, spine and front as the paperback cover file will have them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import { useEffect, useState } from 'react';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { CoverSheetPreview } from '@liminal-hq/folio-protocol/generated/CoverSheetPreview';
import type { Details } from '@liminal-hq/folio-protocol/generated/Details';
import type { BooksClient } from '../books/booksClient';
import { t } from '../i18n/messages';
import styles from './CoverSheetDialog.module.css';

interface CoverSheetDialogProps {
	open: boolean;
	onClose(): void;
	client: BooksClient;
	id: string;
	/** Changes when the book does, so the sheet is drawn again. */
	revision: number;
	/** The book, whose title, author and description are the words on the sheet. */
	book: Book;
	/** Keeps new words for the sheet as the book's details (one undo step). */
	onSave(details: Details): void;
	/** Puts the book's description on the back cover, in the frame that holds it (one undo step). */
	onPutDescription(text: string): void;
}

export function CoverSheetDialog({
	open,
	onClose,
	client,
	id,
	revision,
	book,
	onSave,
	onPutDescription,
}: CoverSheetDialogProps) {
	// The back cover follows the description until its words are changed, or the frame is deleted.
	const description = book.metadata.description.trim();
	const onCover = book.spreads[0]?.frames.find((f) => f.id === 'blurb');
	const heldText =
		onCover?.type === 'text' ? (onCover.content?.inline ?? []).map((r) => r.text).join('') : null;
	const detached = description !== '' && heldText?.trim() !== description;
	const [words, setWords] = useState(() => wordsOf(book));
	useEffect(() => {
		if (open) setWords(wordsOf(book));
		// The fields start from the book each time the sheet opens, not as the book changes under them.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open]);
	const changed = JSON.stringify(words) !== JSON.stringify(wordsOf(book));
	const keep = () => {
		const [first, ...others] = book.authors;
		const name = words.author.trim();
		onSave({
			title: words.title.trim() || book.title,
			subtitle: book.subtitle ?? null,
			authors: name ? [{ name, role: first?.role ?? 'author' }, ...others] : others,
			description: words.description.trim(),
			isbn: book.metadata.isbn,
		});
	};
	const [picture, setPicture] = useState<CoverSheetPreview | null | undefined>(undefined);
	const [guides, setGuides] = useState(true);
	const [failed, setFailed] = useState(false);
	useEffect(() => {
		if (!open) return;
		let live = true;
		// The picture on show stays until the new one is drawn, so a change does not blink the dialog.
		setFailed(false);
		client
			.previewCoverSheet(id)
			.then((found) => live && setPicture(found))
			.catch(() => live && setFailed(true));
		return () => {
			live = false;
		};
	}, [open, client, id, revision]);

	return (
		<Dialog
			open={open}
			title={t('coverSheet.title')}
			size="large"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton variant="primary" onClick={onClose}>
						{t('coverSheet.done')}
					</DialogButton>
				</DialogActions>
			}
		>
			<p className={styles.hint}>{t('coverSheet.hint')}</p>
			{failed && (
				<p role="alert" className={styles.problem}>
					{t('coverSheet.failed')}
				</p>
			)}
			{picture === undefined && !failed && (
				// A box the shape of the sheet, so the dialog is its full size before the picture arrives.
				<div
					role="status"
					className={styles.waiting}
					style={{ aspectRatio: `${2 * book.trim.w + 12} / ${book.trim.h + 18}` }}
				>
					{t('coverSheet.drawing')}
				</div>
			)}
			{picture === null && <p className={styles.hint}>{t('coverSheet.none')}</p>}
			{picture && (
				<label className={styles.toggle}>
					<input
						type="checkbox"
						checked={guides}
						onChange={(event) => setGuides(event.target.checked)}
					/>
					{t('coverSheet.guides')}
				</label>
			)}
			{picture && (
				<div className={styles.stage}>
					<img className={styles.sheet} src={picture.image} alt={t('coverSheet.alt')} />
					{guides && <Guides sheet={picture} />}
				</div>
			)}
			{picture && (
				<div className={styles.words}>
					<label className={styles.field}>
						<span>{t('coverSheet.titleField')}</span>
						<input
							type="text"
							value={words.title}
							onChange={(event) => setWords({ ...words, title: event.target.value })}
						/>
					</label>
					<label className={styles.field}>
						<span>{t('coverSheet.authorField')}</span>
						<input
							type="text"
							value={words.author}
							onChange={(event) => setWords({ ...words, author: event.target.value })}
						/>
					</label>
					<label className={styles.field}>
						<span>{t('coverSheet.blurbField')}</span>
						<textarea
							rows={4}
							value={words.description}
							onChange={(event) => setWords({ ...words, description: event.target.value })}
						/>
					</label>
					{detached && (
						<div>
							<DialogButton onClick={() => onPutDescription(description)}>
								{t('coverSheet.putDescription')}
							</DialogButton>
						</div>
					)}
					<div>
						<DialogButton variant="primary" disabled={!changed} onClick={keep}>
							{t('coverSheet.update')}
						</DialogButton>
					</div>
				</div>
			)}
		</Dialog>
	);
}

/** The words the sheet carries, as the form holds them. */
function wordsOf(book: Book) {
	return {
		title: book.title,
		author: book.authors[0]?.name ?? '',
		description: book.metadata.description,
	};
}

/** Points from the edge of a trimmed cover the safe area keeps text and pictures that need to stay. */
const SAFE = 18;
/** The same beside each fold of the spine. */
const SPINE_SAFE = 4.5;

/** The folds, the safe areas and the barcode box over the sheet, in the sheet's own points. */
function Guides({ sheet }: { sheet: CoverSheetPreview }) {
	const { width, height, edge, cover, spine, barcode } = sheet;
	const backRight = edge + cover;
	const frontLeft = backRight + spine;
	const safe = (x: number, w: number, key: string) => (
		<rect
			key={key}
			x={x}
			y={edge + SAFE}
			width={w}
			height={height - 2 * (edge + SAFE)}
			className={styles.safe}
		/>
	);
	return (
		<svg
			className={styles.guides}
			viewBox={`0 0 ${width} ${height}`}
			preserveAspectRatio="none"
			aria-hidden="true"
			data-testid="cover-guides"
		>
			{safe(edge + SAFE, cover - 2 * SAFE, 'back')}
			{safe(frontLeft + SAFE, cover - 2 * SAFE, 'front')}
			{spine > 2 * SPINE_SAFE && safe(backRight + SPINE_SAFE, spine - 2 * SPINE_SAFE, 'spine')}
			{[backRight, frontLeft].map((x) => (
				<line key={x} x1={x} x2={x} y1={0} y2={height} className={styles.spineFold} />
			))}
			<rect
				x={barcode.x}
				y={barcode.y}
				width={barcode.w}
				height={barcode.h}
				className={styles.barcode}
			/>
		</svg>
	);
}
