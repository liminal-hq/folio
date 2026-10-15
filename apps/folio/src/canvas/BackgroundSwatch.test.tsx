// Tests for the small sample of a background
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Background } from '@liminal-hq/folio-protocol/generated/Background';
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { BackgroundSwatch } from './BackgroundSwatch';

const draw = (bg: Background | null, images = {}) =>
	render(<BackgroundSwatch bg={bg} images={images} />).container.firstElementChild as HTMLElement;

describe('BackgroundSwatch', () => {
	it('is an empty outline for the bare paper', () => {
		const swatch = draw(null);
		expect(swatch.getAttribute('data-empty')).toBe('true');
		expect(swatch.children).toHaveLength(0);
	});

	it('shows a colour and a gradient as CSS', () => {
		expect(
			draw({ type: 'colour', colour: '#112233' }).querySelector('span')!.style.background,
		).toMatch(/#112233|rgb\(17, 34, 51\)/);
		const gradient = draw({
			type: 'gradient',
			angle: 180,
			stops: [
				{ at: 0, colour: '#000000' },
				{ at: 1, colour: '#ffffff' },
			],
		});
		expect(gradient.querySelector('span')!.style.background).toContain('linear-gradient');
	});

	it('shows a texture as a tile, and a picture once it has been read', () => {
		expect(draw({ type: 'texture', id: 'linen' }).querySelector('pattern')).not.toBeNull();
		const picture: Background = {
			type: 'picture',
			asset: 'assets/images/a.jpg',
			fit: 'fill',
			focus: [0.5, 0.5],
		};
		expect(draw(picture).querySelector('span')).toBeNull();
		expect(
			draw(picture, { 'assets/images/a.jpg': 'blob:a' }).querySelector('span')!.style.background,
		).toContain('blob:a');
	});

	it('is decoration, hidden from assistive technology', () => {
		expect(draw(null).getAttribute('aria-hidden')).toBe('true');
	});
});
