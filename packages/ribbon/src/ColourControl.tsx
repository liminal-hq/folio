// A ribbon control that shows a colour chip and opens the colour picker under it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { ColourPicker } from '@liminal-hq/folio-chrome/ColourPicker/ColourPicker';
import { ColourPopover } from '@liminal-hq/folio-chrome/ColourPicker/ColourPopover';
import { useRef, useState } from 'react';
import { Glyph, type ControlProps } from './RibbonControls';
import styles from './Ribbon.module.css';

/**
 * The button keeps the focus where it was on a press, like every ribbon button, so the words being
 * edited stay selected. The popover opens with the focus left alone for a pointer, and moved into
 * the picker for a key. Choosing a colour closes it, returning the focus to this button first when
 * it was inside the picker (so the app can hand it back to the words), then reports the colour.
 */
export function ColourControl(props: ControlProps) {
	const { item, showLabel, tooltip, value, onValue } = props;
	const [open, setOpen] = useState(false);
	const byKey = useRef(false);
	const button = useRef<HTMLButtonElement>(null);
	const colour = item.colour;
	const large = item.size === 'large';
	const set = typeof value === 'string' && value !== '' ? value : null;

	const closeKeepingFocus = () => {
		const inside = document.activeElement?.closest('[role="dialog"][data-keeps-editor]');
		setOpen(false);
		if (inside) button.current?.focus();
	};

	return (
		<>
			<button
				ref={button}
				type="button"
				className={`${styles.button} ${large ? styles.large : styles.small}`}
				aria-label={item.label}
				aria-haspopup="dialog"
				aria-expanded={open}
				disabled={item.disabled || !colour}
				onMouseDown={(e) => {
					tooltip.hide();
					e.preventDefault();
				}}
				onClick={(e) => {
					tooltip.hide();
					byKey.current = e.detail === 0;
					setOpen((was) => !was);
				}}
				onPointerEnter={(e) => tooltip.show(item, e.currentTarget)}
				onPointerLeave={tooltip.hide}
				onFocus={(e) => tooltip.show(item, e.currentTarget)}
				onBlur={tooltip.hide}
			>
				<Glyph d={item.d} px={large ? 22 : 14} swatch={item.swatch} />
				{showLabel && <span className={styles.label}>{item.label}</span>}
				<span className={styles.caret} aria-hidden="true">
					▾
				</span>
			</button>
			{open && colour && button.current && (
				<ColourPopover anchor={button.current} label={item.label} onClose={() => setOpen(false)}>
					<ColourPicker
						{...colour}
						value={set}
						focusOnOpen={byKey.current}
						onChange={(next, final) => {
							if (final) closeKeepingFocus();
							onValue(item, next);
						}}
						onReset={() => {
							closeKeepingFocus();
							onValue(item, '');
						}}
						onNone={
							colour.allowNone
								? () => {
										closeKeepingFocus();
										onValue(item, '');
									}
								: undefined
						}
						onDone={() => setOpen(false)}
					/>
				</ColourPopover>
			)}
		</>
	);
}
