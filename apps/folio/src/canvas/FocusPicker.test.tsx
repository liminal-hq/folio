// Tests for pointing at the part of a picture to keep in view
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { FocusPicker } from './FocusPicker';

const setup = (focus: [number, number] = [0.5, 0.5], disabled = false) => {
	const onChange = vi.fn();
	render(<FocusPicker href="blob:a" focus={focus} onChange={onChange} disabled={disabled} />);
	return { onChange, slider: screen.getByRole('slider') };
};

describe('FocusPicker', () => {
	it('says where the focus is in words', () => {
		const { slider } = setup([0.25, 0.75]);
		expect(slider.getAttribute('aria-valuetext')).toBe('25% across, 75% down');
	});

	it('moves by the arrow keys and stops at the edges', async () => {
		const { onChange, slider } = setup([0.98, 0]);
		slider.focus();
		await userEvent.keyboard('{ArrowRight}{ArrowUp}{ArrowDown}');
		expect(onChange).toHaveBeenNthCalledWith(1, [1, 0]);
		expect(onChange).toHaveBeenNthCalledWith(2, [0.98, 0]);
		expect(onChange).toHaveBeenNthCalledWith(3, [0.98, expect.closeTo(0.05)]);
	});

	it('takes the point clicked, as a share of the picture', () => {
		const { onChange, slider } = setup();
		slider.getBoundingClientRect = () => ({ left: 10, top: 20, width: 100, height: 50 }) as DOMRect;
		fireEvent.pointerDown(slider, { clientX: 35, clientY: 45, pointerId: 1 });
		expect(onChange).toHaveBeenCalledWith([0.25, 0.5]);
	});

	it('does nothing when disabled', async () => {
		const { onChange, slider } = setup([0.5, 0.5], true);
		fireEvent.pointerDown(slider, { clientX: 0, clientY: 0, pointerId: 1 });
		slider.focus();
		await userEvent.keyboard('{ArrowRight}');
		expect(onChange).not.toHaveBeenCalled();
	});
});
