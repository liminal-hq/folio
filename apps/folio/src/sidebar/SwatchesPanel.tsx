// The Swatches panel: the colours a book names, with the exact ink each prints as
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { BookSwatch } from '@liminal-hq/folio-protocol/generated/BookSwatch';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { useState } from 'react';
import { inkToHex, newSwatchId } from '../books/bookSwatches';
import { t } from '../i18n/messages';
import styles from './SwatchesPanel.module.css';

interface SwatchesPanelProps {
	book: Book;
	apply(intent: Intent): void;
}

const PLATES = ['c', 'm', 'y', 'k'] as const;
type Plate = (typeof PLATES)[number];

/** What the form holds: a name and a percent for each plate, as text so a half-typed number is fine. */
interface Form {
	/** The swatch being changed, or `null` for a new one. */
	id: string | null;
	name: string;
	plates: Record<Plate, string>;
}

const BLANK: Form = { id: null, name: '', plates: { c: '0', m: '0', y: '0', k: '0' } };

const percent = (text: string): number | null => {
	if (text.trim() === '') return null;
	const n = Number(text);
	return Number.isInteger(n) && n >= 0 && n <= 100 ? n : null;
};

export function SwatchesPanel({ book, apply }: SwatchesPanelProps) {
	const swatches = book.styles?.swatches ?? [];
	const [form, setForm] = useState<Form>(BLANK);
	const values = PLATES.map((p) => percent(form.plates[p]));
	const valid = form.name.trim() !== '' && values.every((v) => v !== null);
	const ink = valid ? { c: values[0]!, m: values[1]!, y: values[2]!, k: values[3]! } : null;

	const save = () => {
		if (!ink) return;
		const id = form.id ?? newSwatchId(book, form.name);
		const swatch: BookSwatch = {
			id,
			name: form.name.trim(),
			// Rust works the colour a swatch with ink shows as out from the ink.
			display: inkToHex(ink),
			ink,
		};
		apply({ type: 'SetSwatch', swatch });
		setForm(BLANK);
	};

	const edit = (swatch: BookSwatch) =>
		setForm({
			id: swatch.id,
			name: swatch.name,
			plates: {
				c: String(swatch.ink?.c ?? 0),
				m: String(swatch.ink?.m ?? 0),
				y: String(swatch.ink?.y ?? 0),
				k: String(swatch.ink?.k ?? 0),
			},
		});

	return (
		<div className={styles.panel}>
			<p className={styles.hint}>{t('swatches.hint')}</p>
			{swatches.length === 0 && <p className={styles.hint}>{t('swatches.none')}</p>}
			<ul className={styles.list}>
				{swatches.map((swatch) => (
					<li key={swatch.id} className={styles.row}>
						<span
							className={styles.chip}
							style={{ background: swatch.display }}
							aria-hidden="true"
						/>
						<span className={styles.text}>
							<span className={styles.name}>{swatch.name}</span>
							<span className={styles.sub}>
								{swatch.ink
									? `C${swatch.ink.c} M${swatch.ink.m} Y${swatch.ink.y} K${swatch.ink.k}`
									: swatch.display}
							</span>
						</span>
						<button
							type="button"
							className={styles.button}
							aria-label={t('swatches.edit', { name: swatch.name })}
							onClick={() => edit(swatch)}
						>
							{t('swatches.editShort')}
						</button>
						<button
							type="button"
							className={styles.button}
							aria-label={t('swatches.remove', { name: swatch.name })}
							onClick={() => apply({ type: 'RemoveSwatch', id: swatch.id })}
						>
							{t('swatches.removeShort')}
						</button>
					</li>
				))}
			</ul>
			<form
				className={styles.form}
				aria-label={form.id ? t('swatches.formEdit') : t('swatches.formAdd')}
				onSubmit={(event) => {
					event.preventDefault();
					save();
				}}
			>
				<label className={styles.field}>
					<span>{t('swatches.name')}</span>
					<input
						value={form.name}
						onChange={(event) => setForm({ ...form, name: event.target.value })}
					/>
				</label>
				<div className={styles.plates}>
					{PLATES.map((plate) => (
						<label key={plate} className={styles.field}>
							<span>{t(`swatches.plate.${plate}`)}</span>
							<input
								inputMode="numeric"
								value={form.plates[plate]}
								aria-invalid={percent(form.plates[plate]) === null}
								onChange={(event) =>
									setForm({ ...form, plates: { ...form.plates, [plate]: event.target.value } })
								}
							/>
						</label>
					))}
					<span
						className={styles.preview}
						style={{ background: ink ? inkToHex(ink) : 'transparent' }}
						aria-hidden="true"
					/>
				</div>
				<p className={styles.hint}>{t('swatches.screenHint')}</p>
				<div className={styles.actions}>
					<button type="submit" className={styles.primary} disabled={!valid}>
						{form.id ? t('swatches.save') : t('swatches.add')}
					</button>
					{form.id && (
						<button type="button" className={styles.button} onClick={() => setForm(BLANK)}>
							{t('swatches.cancel')}
						</button>
					)}
				</div>
			</form>
		</div>
	);
}
