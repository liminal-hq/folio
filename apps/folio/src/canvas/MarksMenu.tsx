// The stage header's Marks menu: switch each page mark on or off and pick its colour
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useRef, useState } from 'react';
import { t, type MessageKey } from '../i18n/messages';
import { MARK_KEYS, MARK_PALETTES, type MarkColours, type MarkKey, type Marks } from './marksModel';
import styles from './MarksMenu.module.css';

export interface MarksMenuProps {
	marks: Marks;
	colours: MarkColours;
	onMarks(next: Partial<Marks>): void;
	onColour(key: MarkKey, index: number): void;
}

export function MarksMenu({ marks, colours, onMarks, onColour }: MarksMenuProps) {
	const [open, setOpen] = useState(false);
	const root = useRef<HTMLDivElement | null>(null);

	useEffect(() => {
		if (!open) return;
		const away = (event: PointerEvent) => {
			if (!root.current?.contains(event.target as Node)) setOpen(false);
		};
		const escape = (event: KeyboardEvent) => {
			if (event.key !== 'Escape') return;
			// Closing the menu is all this Escape does.
			event.preventDefault();
			setOpen(false);
		};
		document.addEventListener('pointerdown', away);
		document.addEventListener('keydown', escape);
		return () => {
			document.removeEventListener('pointerdown', away);
			document.removeEventListener('keydown', escape);
		};
	}, [open]);

	return (
		<div ref={root} className={styles.root}>
			<button
				type="button"
				className={styles.button}
				aria-haspopup="dialog"
				aria-expanded={open}
				onClick={() => setOpen(!open)}
			>
				{t('marks.button')}
			</button>
			{open && (
				<div className={styles.panel} role="dialog" aria-label={t('marks.menu')}>
					{MARK_KEYS.map((key) => {
						const name = t(`marks.${key}` as MessageKey);
						return (
							<div key={key} className={styles.row}>
								<label className={styles.check}>
									<input
										type="checkbox"
										checked={marks[key]}
										onChange={(event) => onMarks({ [key]: event.target.checked })}
									/>
									{name}
								</label>
								<span
									className={styles.swatches}
									role="group"
									aria-label={`${name}: ${t('marks.colours')}`}
								>
									{MARK_PALETTES[key].map((colour, index) => (
										<button
											key={colour}
											type="button"
											className={styles.swatch}
											style={{ background: colour }}
											aria-label={t('marks.colour', { mark: name, number: index + 1 })}
											aria-pressed={colours[key] === colour}
											onClick={() => onColour(key, index)}
										/>
									))}
								</span>
							</div>
						);
					})}
				</div>
			)}
		</div>
	);
}
