// A swatch grid with a box for any other colour, and a way to choose none
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useState } from 'react';
import type { Swatch } from '@liminal-hq/folio-protocol/generated/Swatch';
import { t } from '../i18n/messages';
import { parseColour } from './colour';
import styles from './ColourPicker.module.css';

export interface ColourPickerProps {
	/** What this chooses, for assistive technology: "Page colour". */
	label: string;
	/** The colour now, as `#rrggbb`, or `null` for none. */
	value: string | null;
	swatches: Swatch[];
	/** Offers "None" for a fill that can be cleared. */
	allowNone?: boolean;
	/** Called with the colour chosen or typed, or `null` for none. */
	onPick(colour: string | null): void;
}

/**
 * Choosing a swatch picks it at once. The box takes `#rgb` or `#rrggbb`: Enter or leaving it keeps
 * a colour that reads and puts the old one back otherwise, so a typo never reaches the book.
 */
export function ColourPicker({ label, value, swatches, allowNone, onPick }: ColourPickerProps) {
	const [draft, setDraft] = useState<string | null>(null);
	const invalid = draft !== null && draft.trim() !== '' && parseColour(draft) === null;

	const finish = () => {
		if (draft === null) return;
		const next = parseColour(draft);
		setDraft(null);
		if (next !== null && next !== value) onPick(next);
	};

	return (
		<div className={styles.picker} role="group" aria-label={label}>
			<div className={styles.grid}>
				{swatches.map((swatch) => (
					<button
						key={swatch.colour}
						type="button"
						className={styles.swatch}
						style={{ background: swatch.colour }}
						aria-label={swatch.name}
						aria-pressed={swatch.colour === value}
						title={swatch.name}
						onClick={() => onPick(swatch.colour)}
					/>
				))}
			</div>
			<div className={styles.row}>
				<input
					type="text"
					className={styles.hex}
					value={draft ?? value ?? ''}
					placeholder="#rrggbb"
					spellCheck={false}
					aria-label={t('colour.custom')}
					aria-invalid={invalid}
					onChange={(event) => setDraft(event.target.value)}
					onBlur={finish}
					onKeyDown={(event) => {
						if (event.key === 'Enter') {
							event.preventDefault();
							finish();
						} else if (event.key === 'Escape' && draft !== null) {
							event.preventDefault();
							event.stopPropagation();
							setDraft(null);
						}
					}}
				/>
				{allowNone && (
					<button
						type="button"
						className={styles.none}
						aria-pressed={value === null}
						onClick={() => onPick(null)}
					>
						{t('colour.none')}
					</button>
				)}
			</div>
			{invalid && (
				<p className={styles.error} role="alert">
					{t('colour.invalid')}
				</p>
			)}
		</div>
	);
}
