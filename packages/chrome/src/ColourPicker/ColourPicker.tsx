// The colour picker: a Swatches tab for the colours a book and Folio offer, and a Custom tab for any other
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import { parseColour } from './colourMaths';
import styles from './ColourPicker.module.css';
import { CustomTab } from './CustomTab';
import { SwatchesTab } from './SwatchesTab';
import {
	defaultColourPickerLabels,
	type ColourContrast,
	type ColourOption,
	type ColourPickerLabels,
} from './types';

export interface ColourPickerProps {
	/** The colour set here: `#rrggbb` or a reference such as `swatch:<id>`; `null` when it is inherited or none. */
	value: string | null;
	/** The colour it falls back to ("From style"). With it, the Swatches tab offers a Reset. */
	inherited?: ColourOption | null;
	/** The book's own swatches, named, with ink where they have some. */
	swatches: ColourOption[];
	/** Pure black and white, always offered first. */
	pinned: ColourOption[];
	/** The app's colours, greys and then its palette. */
	theme: ColourOption[];
	/** The colours used last, as `#rrggbb`, most recent first. */
	recents: string[];
	/** The colour of the page, for the contrast hint of the Custom tab. */
	pageColour?: string;
	/** Whether to show contrast against the page, and the threshold to judge it by. */
	contrast?: ColourContrast;
	/** Easy shows the square, hue and hex; Expert adds RGB. */
	mode: 'easy' | 'expert';
	/** Offers "None" for a fill that can be cleared. */
	allowNone?: boolean;
	disabled?: boolean;
	labels?: Partial<ColourPickerLabels>;
	/** Which tab opens first. */
	initialTab?: 'swatches' | 'custom';
	/** Move focus into the picker on open (a keyboard opener); a pointer opener leaves it where it was. */
	focusOnOpen?: boolean;
	/** A colour is meant. `final` is true for a choice that ends the picking: a swatch, a recent, Enter. */
	onChange(value: string, final: boolean): void;
	onNone?(): void;
	onReset?(): void;
	/** A final choice was made: the owner can close the popover. */
	onDone?(): void;
	/** A colour was used and belongs in the recents: called at once for a final choice, and as the picker closes for a refined one. */
	onRecord?(hex: string): void;
}

type Tab = 'swatches' | 'custom';

export function ColourPicker(props: ColourPickerProps) {
	const labels = useMemo(() => ({ ...defaultColourPickerLabels, ...props.labels }), [props.labels]);
	const [tab, setTab] = useState<Tab>(props.initialTab ?? 'swatches');
	const id = useId();
	const tabs = useRef<Record<Tab, HTMLButtonElement | null>>({ swatches: null, custom: null });
	const refined = useRef<string | null>(null);
	const record = useRef(props.onRecord);
	record.current = props.onRecord;

	// A colour refined with the square or sliders is recorded once the picker goes away.
	useEffect(
		() => () => {
			if (refined.current) record.current?.(refined.current);
		},
		[],
	);

	const shownOf = (value: string | null): string | null => {
		if (value === null) return null;
		const known = [...props.swatches, ...props.pinned, ...props.theme].find(
			(o) => o.value === value,
		);
		return known ? (known.shown ?? known.value) : parseColour(value);
	};
	const currentHex = shownOf(props.value);
	const editing =
		currentHex ?? shownOf(props.inherited?.value ?? null) ?? props.inherited?.shown ?? '#000000';

	const pick = (value: string) => {
		const hex = shownOf(value) ?? parseColour(value);
		refined.current = null;
		props.onChange(value, true);
		if (hex) props.onRecord?.(hex);
		props.onDone?.();
	};

	const onTabKey = (event: KeyboardEvent, from: Tab) => {
		if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
		event.preventDefault();
		const next: Tab = from === 'swatches' ? 'custom' : 'swatches';
		setTab(next);
		tabs.current[next]?.focus();
	};

	return (
		<div className={styles.picker} data-colour-picker="">
			<div className={styles.tabs} role="tablist" aria-label={labels.tabs}>
				{(['swatches', 'custom'] as const).map((t) => (
					<button
						key={t}
						ref={(el) => {
							tabs.current[t] = el;
						}}
						type="button"
						role="tab"
						id={`${id}-tab-${t}`}
						className={styles.tab}
						aria-selected={tab === t}
						aria-controls={`${id}-panel-${t}`}
						tabIndex={tab === t ? 0 : -1}
						onMouseDown={(event) => event.preventDefault()}
						onClick={() => setTab(t)}
						onKeyDown={(event) => onTabKey(event, t)}
					>
						{t === 'swatches' ? labels.tabSwatches : labels.tabCustom}
					</button>
				))}
			</div>
			<div role="tabpanel" id={`${id}-panel-${tab}`} aria-labelledby={`${id}-tab-${tab}`}>
				{tab === 'swatches' ? (
					<SwatchesTab
						value={props.value}
						currentHex={currentHex}
						inherited={props.inherited}
						swatches={props.swatches}
						pinned={props.pinned}
						theme={props.theme}
						recents={props.recents}
						allowNone={props.allowNone}
						disabled={props.disabled}
						labels={labels}
						focusFirst={props.focusOnOpen}
						onPick={pick}
						onNone={
							props.onNone
								? () => {
										props.onNone?.();
										props.onDone?.();
									}
								: undefined
						}
						onReset={
							props.onReset
								? () => {
										props.onReset?.();
										props.onDone?.();
									}
								: undefined
						}
					/>
				) : (
					<CustomTab
						hex={editing}
						labels={labels}
						mode={props.mode}
						pageColour={props.pageColour}
						contrast={props.contrast}
						disabled={props.disabled}
						onApply={(hex, final) => {
							props.onChange(hex, final);
							if (final) {
								refined.current = null;
								props.onRecord?.(hex);
								props.onDone?.();
							} else refined.current = hex;
						}}
					/>
				)}
			</div>
		</div>
	);
}
