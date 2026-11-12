// The saturation and brightness square of the Custom tab, driven by pointer or arrow keys
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useRef, type KeyboardEvent, type PointerEvent } from 'react';
import { clamp } from './colourMaths';
import styles from './ColourPicker.module.css';

export interface SaturationSquareProps {
	hue: number;
	saturation: number;
	value: number;
	label: string;
	valueText: string;
	/** Called while the pointer drags or a key steps. */
	onChange(saturation: number, value: number): void;
	/** Called when a drag ends or a key is released: the moment the colour is meant. */
	onCommit(): void;
	disabled?: boolean;
}

const KEY_STEPS: Record<string, [number, number]> = {
	ArrowLeft: [-1, 0],
	ArrowRight: [1, 0],
	ArrowUp: [0, 1],
	ArrowDown: [0, -1],
};

export function SaturationSquare(props: SaturationSquareProps) {
	const { hue, saturation, value, label, valueText, onChange, onCommit, disabled } = props;
	const box = useRef<HTMLDivElement>(null);
	const dragging = useRef(false);

	const fromPointer = (event: PointerEvent) => {
		const rect = box.current!.getBoundingClientRect();
		if (rect.width === 0 || rect.height === 0) return;
		onChange(
			clamp((event.clientX - rect.left) / rect.width, 0, 1),
			1 - clamp((event.clientY - rect.top) / rect.height, 0, 1),
		);
	};

	return (
		<div
			ref={box}
			className={styles.square}
			style={{ backgroundColor: `hsl(${hue} 100% 50%)` }}
			role="slider"
			tabIndex={disabled ? -1 : 0}
			aria-label={label}
			aria-valuemin={0}
			aria-valuemax={100}
			aria-valuenow={Math.round(value * 100)}
			aria-valuetext={valueText}
			aria-disabled={disabled || undefined}
			onPointerDown={(event) => {
				if (disabled) return;
				// The words being edited keep the focus unless the keyboard is in the square.
				event.preventDefault();
				dragging.current = true;
				box.current?.setPointerCapture?.(event.pointerId);
				fromPointer(event);
			}}
			onPointerMove={(event) => dragging.current && fromPointer(event)}
			onPointerUp={() => {
				if (!dragging.current) return;
				dragging.current = false;
				onCommit();
			}}
			onPointerCancel={() => {
				dragging.current = false;
			}}
			onKeyDown={(event: KeyboardEvent) => {
				const step = KEY_STEPS[event.key];
				if (!step || disabled) return;
				event.preventDefault();
				const size = event.shiftKey ? 0.1 : 0.01;
				onChange(clamp(saturation + step[0] * size, 0, 1), clamp(value + step[1] * size, 0, 1));
			}}
			onKeyUp={(event) => {
				if (KEY_STEPS[event.key]) onCommit();
			}}
		>
			<i
				className={styles.knob}
				style={{ left: `${saturation * 100}%`, top: `${(1 - value) * 100}%` }}
			/>
		</div>
	);
}
