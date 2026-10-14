// The ribbon's controls: glyph, button, toggle, menu button, select and stepper
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { CSSProperties, MouseEvent } from 'react';
import type { RibbonLabels } from './labels';
import styles from './Ribbon.module.css';
import type { RibbonTooltip } from './useRibbonTooltip';
import type { RibbonItem, RibbonValue } from './types';

function Glyph({ d, px, swatch }: { d: string; px: number; swatch?: string }) {
	return (
		<span className={styles.glyph}>
			<svg width={px} height={px} viewBox="0 0 16 16" aria-hidden="true" focusable="false">
				<path
					d={d}
					fill="none"
					stroke="currentColor"
					strokeWidth="1.3"
					strokeLinecap="round"
					strokeLinejoin="round"
				/>
			</svg>
			{swatch && (
				<span
					className={styles.swatch}
					style={{ '--folio-ribbon-swatch': swatch } as CSSProperties}
				/>
			)}
		</span>
	);
}

export interface ControlProps {
	item: RibbonItem;
	showLabel: boolean;
	tooltip: RibbonTooltip;
	/** The pressed state of a toggle, resolved by the ribbon. */
	pressed: boolean;
	/** The value of a select or stepper, resolved by the ribbon. */
	value: RibbonValue | undefined;
	labels: RibbonLabels;
	onPress: (item: RibbonItem, anchor: HTMLElement) => void;
	onValue: (item: RibbonItem, value: RibbonValue) => void;
}

/** Keeps focus (and so a text selection) where it was when the pointer goes down on a button. */
function keepFocus(event: MouseEvent) {
	event.preventDefault();
}

export function RibbonControl(props: ControlProps) {
	const { item, showLabel, tooltip, pressed, value, labels, onPress, onValue } = props;
	const kind = item.kind ?? 'button';
	const tipHandlers = {
		onPointerEnter: (e: React.PointerEvent<HTMLElement>) => tooltip.show(item, e.currentTarget),
		onPointerLeave: tooltip.hide,
		onFocus: (e: React.FocusEvent<HTMLElement>) => tooltip.show(item, e.currentTarget),
		onBlur: tooltip.hide,
	};

	if (kind === 'select') {
		return (
			<select
				className={styles.select}
				style={
					item.width
						? ({ '--folio-ribbon-select-width': `${item.width}px` } as CSSProperties)
						: undefined
				}
				aria-label={item.label}
				disabled={item.disabled}
				value={value === undefined ? '' : String(value)}
				onChange={(e) => onValue(item, e.target.value)}
				{...tipHandlers}
			>
				{(item.options ?? []).map(([v, l]) => (
					<option key={v} value={v}>
						{l}
					</option>
				))}
			</select>
		);
	}

	if (kind === 'stepper') {
		const current = typeof value === 'number' ? value : Number(item.min ?? 0);
		const step = item.step ?? 1;
		const clamp = (n: number) => Math.min(item.max ?? Infinity, Math.max(item.min ?? -Infinity, n));
		return (
			<span className={styles.stepper} role="group" aria-label={item.label}>
				<button
					type="button"
					className={styles.stepperButton}
					aria-label={`${labels.decrease} ${item.label}`}
					disabled={item.disabled || current <= (item.min ?? -Infinity)}
					onMouseDown={keepFocus}
					onClick={() => onValue(item, clamp(current - step))}
				>
					−
				</button>
				<output className={styles.stepperValue} aria-label={item.label}>
					{current}
				</output>
				<button
					type="button"
					className={styles.stepperButton}
					aria-label={`${labels.increase} ${item.label}`}
					disabled={item.disabled || current >= (item.max ?? Infinity)}
					onMouseDown={keepFocus}
					onClick={() => onValue(item, clamp(current + step))}
				>
					+
				</button>
			</span>
		);
	}

	const large = item.size === 'large';
	return (
		<button
			type="button"
			className={`${styles.button} ${large ? styles.large : styles.small}`}
			aria-label={item.label}
			aria-pressed={kind === 'toggle' ? pressed : undefined}
			aria-haspopup={kind === 'menu' ? 'menu' : undefined}
			data-pressed={kind === 'toggle' && pressed ? '' : undefined}
			disabled={item.disabled}
			onMouseDown={keepFocus}
			onClick={(e) => onPress(item, e.currentTarget)}
			{...tipHandlers}
		>
			<Glyph d={item.d} px={large ? 22 : 14} swatch={item.swatch} />
			{showLabel && <span className={styles.label}>{item.label}</span>}
			{kind === 'menu' && (
				<span className={styles.caret} aria-hidden="true">
					▾
				</span>
			)}
		</button>
	);
}
