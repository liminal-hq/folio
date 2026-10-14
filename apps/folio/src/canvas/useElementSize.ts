// The size of an element, kept current as it resizes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useRef, useState, type RefObject } from 'react';

export interface ElementSize {
	w: number;
	h: number;
}

export function useElementSize<T extends HTMLElement>(): [RefObject<T | null>, ElementSize] {
	const ref = useRef<T | null>(null);
	const [size, setSize] = useState<ElementSize>({ w: 0, h: 0 });

	useEffect(() => {
		const el = ref.current;
		if (el === null) return;
		const read = () => setSize({ w: el.clientWidth, h: el.clientHeight });
		read();
		if (typeof ResizeObserver === 'undefined') return;
		const observer = new ResizeObserver(read);
		observer.observe(el);
		return () => observer.disconnect();
	}, []);

	return [ref, size];
}
