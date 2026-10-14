// The ribbon's own tooltip: slow to appear, quick when moving between buttons
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import {
	useCallback,
	useEffect,
	useRef,
	useState,
	type CSSProperties,
	type ReactNode,
} from 'react';
import styles from './Ribbon.module.css';
import type { RibbonItem } from './types';

/** How long the pointer rests on a control before its tooltip appears. */
export const TOOLTIP_DELAY_MS = 450;
/** The delay while moving from one control to the next. */
export const TOOLTIP_WARM_DELAY_MS = 60;
/** How soon after one tooltip closes the next counts as moving between controls. */
const WARM_WINDOW_MS = 400;

interface Shown {
	item: RibbonItem;
	x: number;
	y: number;
}

export interface RibbonTooltip {
	show: (item: RibbonItem, anchor: HTMLElement) => void;
	hide: () => void;
	node: ReactNode;
}

export function useRibbonTooltip(): RibbonTooltip {
	const [shown, setShown] = useState<Shown | null>(null);
	const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
	const lastClosed = useRef(Number.NEGATIVE_INFINITY);
	const open = useRef(false);

	const clear = useCallback(() => {
		if (timer.current !== null) clearTimeout(timer.current);
		timer.current = null;
	}, []);

	const hide = useCallback(() => {
		clear();
		if (open.current) lastClosed.current = Date.now();
		open.current = false;
		setShown(null);
	}, [clear]);

	const show = useCallback(
		(item: RibbonItem, anchor: HTMLElement) => {
			clear();
			const warm = open.current || Date.now() - lastClosed.current < WARM_WINDOW_MS;
			timer.current = setTimeout(
				() => {
					const rect = anchor.getBoundingClientRect();
					open.current = true;
					setShown({ item, x: rect.left + rect.width / 2, y: rect.bottom + 6 });
				},
				warm ? TOOLTIP_WARM_DELAY_MS : TOOLTIP_DELAY_MS,
			);
		},
		[clear],
	);

	useEffect(() => clear, [clear]);

	const node = shown && (
		<div
			role="tooltip"
			className={styles.tooltip}
			style={
				{
					'--folio-ribbon-tip-x': `${shown.x}px`,
					'--folio-ribbon-tip-y': `${shown.y}px`,
				} as CSSProperties
			}
		>
			<span className={styles.tooltipName}>
				{shown.item.label}
				{shown.item.key && <kbd className={styles.tooltipKey}>{shown.item.key}</kbd>}
			</span>
			{shown.item.desc && <span className={styles.tooltipDesc}>{shown.item.desc}</span>}
		</div>
	);

	return { show, hide, node };
}
