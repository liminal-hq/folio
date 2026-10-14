// A number box for a length: shown in the person's units, kept in points
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useState } from 'react';
import styles from './Inspector.module.css';
import { fromPoints, toPoints, unitLabel, type Units } from './units';

export interface LengthFieldProps {
	label: string;
	/** The length, in points. */
	points: number;
	units: Units;
	disabled?: boolean;
	/** Commits a length typed and confirmed, in points. */
	onCommit(points: number): void;
}

/**
 * Enter or leaving the box keeps what was typed when it is a number that differs from the length;
 * Escape puts the length back. Anything else is put back too, so a typo never reaches the book.
 */
export function LengthField({ label, points, units, disabled, onCommit }: LengthFieldProps) {
	const shown = String(fromPoints(points, units));
	const [draft, setDraft] = useState<string | null>(null);

	const finish = () => {
		if (draft === null) return;
		const next = toPoints(draft, units);
		setDraft(null);
		if (
			next !== null &&
			Math.abs(next - points) > 1e-6 &&
			fromPoints(next, units) !== Number(shown)
		) {
			onCommit(next);
		}
	};

	return (
		<label className={styles.length}>
			<span>{label}</span>
			<input
				type="text"
				inputMode="decimal"
				value={draft ?? shown}
				disabled={disabled}
				aria-label={`${label} (${unitLabel(units)})`}
				onChange={(event) => setDraft(event.target.value)}
				onBlur={finish}
				onKeyDown={(event) => {
					if (event.key === 'Enter') {
						event.preventDefault();
						finish();
					} else if (event.key === 'Escape') {
						event.preventDefault();
						event.stopPropagation();
						setDraft(null);
					}
				}}
			/>
		</label>
	);
}
