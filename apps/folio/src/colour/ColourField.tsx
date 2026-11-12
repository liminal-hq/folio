// A colour field for panels and dialogs: a chip and the colour's name or hex, opening the colour picker
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { ColourPicker } from '@liminal-hq/folio-chrome/ColourPicker/ColourPicker';
import { ColourPopover } from '@liminal-hq/folio-chrome/ColourPicker/ColourPopover';
import { parseColour } from '@liminal-hq/folio-chrome/ColourPicker/colourMaths';
import { useMemo, useRef, useState } from 'react';
import type { PickerSwatch } from '../books/bookSwatches';
import { t } from '../i18n/messages';
import { useSettings } from '../settings/useSettings';
import styles from './ColourField.module.css';
import { colourName, colourPickerLabels, pinnedColours, themeColours } from './colourPicker';
import { useRecentColours } from './recentColours';

export interface ColourFieldProps {
	/** What this chooses, for assistive technology: "Page colour". */
	label: string;
	/** The colour now, as `#rrggbb` or `swatch:<id>`, or `null` for none. */
	value: string | null;
	/** The swatches of the app and the book, as the picker lists them. */
	swatches: PickerSwatch[];
	/** Offers "None" for a fill that can be cleared. */
	allowNone?: boolean;
	/** The colour of the page, for the picker's contrast hint against it. */
	pageColour?: string;
	/** Called with the colour chosen or typed, or `null` for none. */
	onPick(colour: string | null): void;
	/** Shows the choice but changes nothing, as for a locked frame. */
	disabled?: boolean;
}

/** The chip and name of the colour in use; pressing it opens the picker under it. */
export function ColourField(props: ColourFieldProps) {
	const { label, value, swatches, allowNone, pageColour, onPick, disabled } = props;
	const settings = useSettings();
	const { recents, record } = useRecentColours();
	const [open, setOpen] = useState(false);
	const byKey = useRef(false);
	const button = useRef<HTMLButtonElement>(null);
	const options = useMemo(
		() => swatches.map((s) => ({ value: s.colour, name: s.name, shown: s.shown })),
		[swatches],
	);
	const labels = useMemo(() => colourPickerLabels(), []);

	const named = options.find((o) => o.value === value);
	const hex = value === null ? null : (named?.shown ?? parseColour(value) ?? value);
	const name = named?.name ?? (hex ? colourName(hex) : null);

	const close = () => {
		const inside = document.activeElement?.closest('[role="dialog"][data-keeps-editor]');
		setOpen(false);
		if (inside) button.current?.focus();
	};

	return (
		<>
			<button
				ref={button}
				type="button"
				className={styles.field}
				aria-label={label}
				aria-haspopup="dialog"
				aria-expanded={open}
				disabled={disabled}
				onClick={(e) => {
					byKey.current = e.detail === 0;
					setOpen((was) => !was);
				}}
			>
				<span
					className={styles.chip}
					style={hex ? { background: hex } : undefined}
					data-none={hex ? undefined : ''}
					aria-hidden="true"
				/>
				<span className={styles.text}>{value === null ? t('colour.none') : (name ?? hex)}</span>
			</button>
			{open && button.current && (
				<ColourPopover
					anchor={button.current}
					label={t('colour.picker.dialog', { name: label })}
					onClose={() => setOpen(false)}
				>
					<ColourPicker
						value={value}
						swatches={options}
						pinned={pinnedColours()}
						theme={themeColours()}
						recents={recents}
						pageColour={pageColour}
						contrast={pageColour ? 'graphic' : undefined}
						mode={settings.mode}
						allowNone={allowNone}
						labels={labels}
						focusOnOpen={byKey.current}
						onChange={(next, final) => {
							if (final) close();
							onPick(next);
						}}
						onNone={() => {
							close();
							onPick(null);
						}}
						onDone={() => setOpen(false)}
						onRecord={record}
					/>
				</ColourPopover>
			)}
		</>
	);
}
