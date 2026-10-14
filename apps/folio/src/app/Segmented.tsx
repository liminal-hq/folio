// A segmented choice between a few options, as a radio group
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useRef, type KeyboardEvent } from 'react';
import styles from './Segmented.module.css';

export interface SegmentedOption<T extends string> {
	value: T;
	label: string;
}

interface SegmentedProps<T extends string> {
	label: string;
	options: readonly SegmentedOption<T>[];
	value: T;
	onChange: (value: T) => void;
}

/**
 * A `radiogroup`: the chosen option is the one tab stop, and the arrow keys move the choice, as in
 * a native radio group.
 */
export function Segmented<T extends string>({
	label,
	options,
	value,
	onChange,
}: SegmentedProps<T>) {
	const buttons = useRef<Array<HTMLButtonElement | null>>([]);

	const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
		const step = event.key === 'ArrowRight' || event.key === 'ArrowDown' ? 1 : -1;
		if (!['ArrowRight', 'ArrowDown', 'ArrowLeft', 'ArrowUp'].includes(event.key)) return;
		event.preventDefault();
		const at = options.findIndex((option) => option.value === value);
		const next = options[(at + step + options.length) % options.length];
		if (!next) return;
		onChange(next.value);
		buttons.current[options.indexOf(next)]?.focus();
	};

	return (
		<div className={styles.group} role="radiogroup" aria-label={label} onKeyDown={onKeyDown}>
			{options.map((option, index) => (
				<button
					key={option.value}
					ref={(element) => {
						buttons.current[index] = element;
					}}
					type="button"
					role="radio"
					aria-checked={option.value === value}
					tabIndex={option.value === value ? 0 : -1}
					className={styles.option}
					onClick={() => onChange(option.value)}
				>
					{option.label}
				</button>
			))}
		</div>
	);
}
