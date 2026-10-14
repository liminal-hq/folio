// The stage's zoom: fit to the stage until the reader picks a level, with the keys that change it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useCallback, useEffect, useState } from 'react';
import { clampZoom, stepZoom } from './geometry';

export interface StageZoom {
	/** 1 is 100%. */
	zoom: number;
	/** True while the zoom follows the stage's size. */
	fitting: boolean;
	zoomIn(): void;
	zoomOut(): void;
	fit(): void;
}

/** `fitted` is the zoom that fits the spread in the stage, which changes as either resizes. */
export function useStageZoom(fitted: number, keys = true): StageZoom {
	const [chosen, setChosen] = useState<number | null>(null);
	const zoom = clampZoom(chosen ?? fitted);

	const zoomIn = useCallback(
		() => setChosen(stepZoom(clampZoom(chosen ?? fitted), 1)),
		[chosen, fitted],
	);
	const zoomOut = useCallback(
		() => setChosen(stepZoom(clampZoom(chosen ?? fitted), -1)),
		[chosen, fitted],
	);
	const fit = useCallback(() => setChosen(null), []);

	useEffect(() => {
		if (!keys) return;
		const onKey = (event: KeyboardEvent) => {
			if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
			if (event.key === '=' || event.key === '+') zoomIn();
			else if (event.key === '-' || event.key === '_') zoomOut();
			else if (event.key === '0') fit();
			else return;
			event.preventDefault();
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	}, [keys, zoomIn, zoomOut, fit]);

	return { zoom, fitting: chosen === null, zoomIn, zoomOut, fit };
}
