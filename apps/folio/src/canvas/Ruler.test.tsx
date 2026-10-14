// Tests for the ruler: ticks in its unit, labels, and the selection's tint
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Ruler } from './Ruler';

const scale = 96 / 72;

describe('Ruler', () => {
	it("draws a tick every step, labelled where it is major, from the spread's zero", () => {
		const { container } = render(
			<Ruler axis="x" unit="inches" scale={scale} origin={40} length={400} />,
		);
		const labels = [...container.querySelectorAll('text')].map((t) => t.textContent);
		expect(labels.slice(0, 3)).toEqual(['0', '0.5', '1']);
		// Zero is 40 px in, and the ruler carries on to the left of it, into the bleed.
		const at = [...container.querySelectorAll('line')].map((l) => Number(l.getAttribute('x1')));
		expect(at).toContain(40);
		expect(Math.min(...at)).toBeLessThan(40);
	});

	it('tints what the selection covers', () => {
		const { container } = render(
			<Ruler
				axis="y"
				unit="points"
				scale={scale}
				origin={20}
				length={300}
				extent={{ from: 30, to: 90 }}
			/>,
		);
		const tint = container.querySelector('[data-extent]')!;
		expect(Number(tint.getAttribute('y'))).toBeCloseTo(20 + 30 * scale);
		expect(Number(tint.getAttribute('height'))).toBeCloseTo(60 * scale);
	});

	it('draws no tint for an empty selection and nothing when it has no length', () => {
		const none = render(
			<Ruler axis="x" unit="points" scale={scale} origin={0} length={200} extent={null} />,
		);
		expect(none.container.querySelector('[data-extent]')).toBeNull();
		const flat = render(<Ruler axis="x" unit="points" scale={scale} origin={0} length={0} />);
		expect(flat.container.querySelectorAll('line')).toHaveLength(0);
	});

	it('runs a vertical ruler down the side with its labels turned', () => {
		const { container } = render(
			<Ruler axis="y" unit="millimetres" scale={scale} origin={10} length={300} />,
		);
		expect(container.querySelector('svg')).toHaveAttribute('data-ruler', 'y');
		expect(container.querySelector('text')!.getAttribute('transform')).toMatch(/^rotate\(-90/);
	});
});
