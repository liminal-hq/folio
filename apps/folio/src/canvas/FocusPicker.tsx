// A small copy of a picture to point at the part that should stay in view when it is cropped
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { KeyboardEvent, PointerEvent } from 'react';
import { t } from '../i18n/messages';
import styles from './FocusPicker.module.css';

const STEP = 0.05;
const clamp = (n: number) => Math.min(1, Math.max(0, n));

/**
 * One slider over two axes: click or drag on the picture, or use the arrow keys, to say which point
 * of it matters. Each move reports the new point at once, so the page shows it as it goes.
 */
export function FocusPicker({
	href,
	focus,
	onChange,
	disabled = false,
}: {
	href: string;
	focus: readonly [number, number];
	onChange(focus: [number, number]): void;
	disabled?: boolean;
}) {
	const point = (event: PointerEvent<HTMLDivElement>): [number, number] => {
		const box = event.currentTarget.getBoundingClientRect();
		return [
			clamp((event.clientX - box.left) / box.width),
			clamp((event.clientY - box.top) / box.height),
		];
	};
	const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
		if (disabled) return;
		event.currentTarget.setPointerCapture?.(event.pointerId);
		onChange(point(event));
	};
	const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
		if (disabled || !event.currentTarget.hasPointerCapture?.(event.pointerId)) return;
		onChange(point(event));
	};
	const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
		const move: Record<string, [number, number]> = {
			ArrowLeft: [-STEP, 0],
			ArrowRight: [STEP, 0],
			ArrowUp: [0, -STEP],
			ArrowDown: [0, STEP],
		};
		const by = move[event.key];
		if (disabled || !by) return;
		event.preventDefault();
		onChange([clamp(focus[0] + by[0]), clamp(focus[1] + by[1])]);
	};
	const across = Math.round(focus[0] * 100);
	const down = Math.round(focus[1] * 100);
	return (
		<div
			className={styles.picker}
			role="slider"
			tabIndex={disabled ? -1 : 0}
			aria-label={t('focus.label')}
			aria-valuemin={0}
			aria-valuemax={100}
			aria-valuenow={across}
			aria-valuetext={t('focus.value', { across, down })}
			aria-disabled={disabled || undefined}
			onPointerDown={onPointerDown}
			onPointerMove={onPointerMove}
			onKeyDown={onKeyDown}
		>
			<img src={href} alt="" draggable={false} />
			<span className={styles.mark} style={{ left: `${across}%`, top: `${down}%` }} />
		</div>
	);
}
