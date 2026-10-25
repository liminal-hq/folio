// The barcode dialog: leave the barcode to KDP or make one from the ISBN, with a price add-on
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { BarcodeChoice } from '@liminal-hq/folio-protocol/generated/BarcodeChoice';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { useEffect, useState } from 'react';
import { t } from '../i18n/messages';
import styles from './BarcodeDialog.module.css';
import { readIsbn } from './isbn';

interface BarcodeDialogProps {
	open: boolean;
	onClose(): void;
	book: Book;
	apply(intent: Intent): void;
}

type Who = 'auto' | 'kdp' | 'own';
type Addon = 'none' | 'noPrice' | 'usd' | 'cad';

/** What a price is as four digits: dollars and cents with the point left out (19.99 is 1999). */
function priceDigits(text: string): string | null {
	const match = /^\s*(\d{1,2})(?:[.,](\d{1,2}))?\s*$/.exec(text);
	if (!match) return null;
	return `${match[1]!.padStart(2, '0')}${(match[2] ?? '').padEnd(2, '0')}`;
}

/** The add-on a choice holds, as a kind and the price it was made from. */
function addonOf(choice: BarcodeChoice): { kind: Addon; price: string } {
	if (choice.kind !== 'own' || !choice.addon) return { kind: 'none', price: '' };
	if (choice.addon === '90000') return { kind: 'noPrice', price: '' };
	const digits = choice.addon.slice(1);
	const price = `${Number(digits.slice(0, 2))}.${digits.slice(2)}`;
	return { kind: choice.addon.startsWith('6') ? 'cad' : 'usd', price };
}

export function BarcodeDialog({ open, onClose, book, apply }: BarcodeDialogProps) {
	const [who, setWho] = useState<Who>('auto');
	const [isbn, setIsbn] = useState('');
	const [addon, setAddon] = useState<Addon>('none');
	const [price, setPrice] = useState('');
	useEffect(() => {
		if (!open) return;
		setWho(book.barcode?.kind ?? 'auto');
		setIsbn(book.metadata.isbn.paperback ?? '');
		const held = addonOf(book.barcode ?? { kind: 'auto' });
		setAddon(held.kind);
		setPrice(held.price);
		// The form starts from the book each time the dialog opens.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open]);

	const reading = readIsbn(isbn);
	const digits = priceDigits(price);
	const priced = addon === 'usd' || addon === 'cad';
	const isbnOk = reading.kind === 'ok' || reading.kind === 'empty';
	const valid = who !== 'own' || (reading.kind === 'ok' && (!priced || digits !== null));

	const choice = (): BarcodeChoice => {
		if (who === 'auto') return { kind: 'auto' };
		if (who === 'kdp') return { kind: 'kdp' };
		const code =
			addon === 'noPrice'
				? '90000'
				: priced && digits
					? `${addon === 'cad' ? '6' : '5'}${digits}`
					: undefined;
		return code ? { kind: 'own', addon: code } : { kind: 'own' };
	};

	const save = () => {
		if (reading.kind === 'ok' && reading.isbn !== (book.metadata.isbn.paperback ?? '')) {
			apply({
				type: 'SetDetails',
				details: {
					title: book.title,
					subtitle: book.subtitle ?? null,
					authors: book.authors,
					description: book.metadata.description,
					isbn: { ...book.metadata.isbn, paperback: reading.isbn },
				},
			});
		}
		apply({ type: 'SetBarcode', choice: choice() });
		onClose();
	};

	return (
		<Dialog
			open={open}
			title={t('barcode.title')}
			size="medium"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton onClick={onClose}>{t('barcode.cancel')}</DialogButton>
					<DialogButton variant="primary" disabled={!valid} onClick={save}>
						{t('barcode.save')}
					</DialogButton>
				</DialogActions>
			}
		>
			<fieldset className={styles.group}>
				<legend className={styles.legend}>{t('barcode.who')}</legend>
				{(['auto', 'kdp', 'own'] as const).map((value) => (
					<label key={value} className={styles.option}>
						<input
							type="radio"
							name="barcode-who"
							checked={who === value}
							onChange={() => setWho(value)}
						/>
						<span className={styles.optionText}>
							<span className={styles.name}>{t(`barcode.who.${value}`)}</span>
							<span className={styles.hint}>{t(`barcode.who.${value}.sub`)}</span>
						</span>
					</label>
				))}
			</fieldset>
			{who !== 'kdp' && (
				<div className={styles.form}>
					<label className={styles.field}>
						<span>{t('barcode.isbn')}</span>
						<input
							value={isbn}
							aria-invalid={!isbnOk}
							onChange={(event) => setIsbn(event.target.value)}
						/>
					</label>
					{reading.kind === 'ok' && reading.isbn !== isbn.replace(/[\s-]/g, '') && (
						<p className={styles.hint} role="status">
							{t('barcode.isbn.converted', { isbn: reading.isbn })}
						</p>
					)}
					{reading.kind === 'checkDigit' && (
						<p className={styles.problem} role="alert">
							{t('barcode.isbn.check')}{' '}
							<button type="button" className={styles.link} onClick={() => setIsbn(reading.fixed)}>
								{t('barcode.isbn.fix', { isbn: reading.fixed })}
							</button>
						</p>
					)}
					{reading.kind === 'bad' && (
						<p className={styles.problem} role="alert">
							{t(`barcode.isbn.${reading.reason}`)}
						</p>
					)}
					{who === 'own' && (
						<>
							<label className={styles.field}>
								<span>{t('barcode.addon')}</span>
								<select value={addon} onChange={(e) => setAddon(e.target.value as Addon)}>
									<option value="none">{t('barcode.addon.none')}</option>
									<option value="noPrice">{t('barcode.addon.noPrice')}</option>
									<option value="usd">{t('barcode.addon.usd')}</option>
									<option value="cad">{t('barcode.addon.cad')}</option>
								</select>
							</label>
							{priced && (
								<label className={styles.field}>
									<span>{t('barcode.price')}</span>
									<input
										value={price}
										aria-invalid={price.trim() !== '' && digits === null}
										placeholder="19.99"
										onChange={(event) => setPrice(event.target.value)}
									/>
								</label>
							)}
						</>
					)}
				</div>
			)}
			<p className={styles.hint}>{t('barcode.preview')}</p>
		</Dialog>
	);
}
