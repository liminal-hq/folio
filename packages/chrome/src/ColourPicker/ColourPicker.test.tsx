// Tests for the colour picker: both tabs, keyboard, validation and the fixed width
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ColourPicker, type ColourPickerProps } from './ColourPicker';
import { ColourPopover } from './ColourPopover';

const pinned = [
	{ value: '#000000', name: 'Black' },
	{ value: '#ffffff', name: 'White' },
];
const theme = [
	{ value: '#1c1b20', name: 'Ink' },
	{ value: '#5b2a4a', name: 'Plum' },
	{ value: '#1f4e5f', name: 'Deep teal' },
];
const swatches = [
	{
		value: 'swatch:orange',
		name: 'Title Orange',
		shown: '#ff4d0d',
		ink: [0, 70, 95, 0] as const,
	},
];

function setup(overrides: Partial<ColourPickerProps> = {}) {
	const props: ColourPickerProps = {
		value: null,
		inherited: { value: '#1c1b20', name: 'Ink' },
		swatches,
		pinned,
		theme,
		recents: ['#e07a5f'],
		pageColour: '#ffffff',
		contrast: 'text',
		mode: 'easy',
		onChange: vi.fn(),
		onReset: vi.fn(),
		onDone: vi.fn(),
		onRecord: vi.fn(),
		...overrides,
	};
	return { props, ...render(<ColourPicker {...props} />) };
}

afterEach(() => {
	delete (globalThis as { EyeDropper?: unknown }).EyeDropper;
});

describe('Swatches tab', () => {
	it('offers pure black and white, the theme colours, the book swatches and the recents', () => {
		setup();
		for (const name of ['Black', 'White', 'Ink', 'Plum', 'Title Orange', 'Recent colour']) {
			expect(
				screen.getAllByRole('button', { name: new RegExp(`^${name}`) }).length,
			).toBeGreaterThan(0);
		}
		expect(screen.getByRole('button', { name: 'Black, #000000' })).toBeInTheDocument();
		expect(screen.getByText('C0 M70 Y95 K0')).toBeInTheDocument();
	});

	it('applies a swatch, records it and is done', async () => {
		const { props } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Black, #000000' }));
		expect(props.onChange).toHaveBeenCalledWith('#000000', true);
		expect(props.onRecord).toHaveBeenCalledWith('#000000');
		expect(props.onDone).toHaveBeenCalled();
	});

	it('applies a book swatch by its reference and records the colour it shows as', async () => {
		const { props } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Title Orange, #ff4d0d' }));
		expect(props.onChange).toHaveBeenCalledWith('swatch:orange', true);
		expect(props.onRecord).toHaveBeenCalledWith('#ff4d0d');
	});

	it('resets to the colour from style only when one is set here', async () => {
		const { props, rerender } = setup();
		expect(screen.getByRole('button', { name: 'Reset' })).toBeDisabled();
		rerender(<ColourPicker {...props} value="#5b2a4a" />);
		await userEvent.click(screen.getByRole('button', { name: 'Reset' }));
		expect(props.onReset).toHaveBeenCalled();
	});

	it('marks the colour in use', () => {
		setup({ value: '#5b2a4a' });
		expect(screen.getByRole('button', { name: 'Plum, #5b2a4a' })).toHaveAttribute(
			'aria-pressed',
			'true',
		);
		expect(screen.getByRole('button', { name: 'Black, #000000' })).toHaveAttribute(
			'aria-pressed',
			'false',
		);
	});

	it('moves with the arrow keys, applies on Enter, and keeps one tab stop', async () => {
		const user = userEvent.setup();
		const { props } = setup({ value: '#000000' });
		const stops = screen
			.getAllByRole('button')
			.filter((b) => b.getAttribute('aria-pressed') !== null && b.tabIndex === 0);
		expect(stops).toHaveLength(1);
		screen.getByRole('button', { name: 'Black, #000000' }).focus();
		await user.keyboard('{ArrowRight}');
		expect(screen.getByRole('button', { name: 'White, #ffffff' })).toHaveFocus();
		await user.keyboard('{ArrowDown}');
		// Down keeps the column: the second of the pinned pair sits over the second theme colour.
		expect(screen.getByRole('button', { name: 'Plum, #5b2a4a' })).toHaveFocus();
		await user.keyboard('{ArrowRight}{Enter}');
		expect(props.onChange).toHaveBeenCalledWith('#1f4e5f', true);
	});

	it('names the colour under focus in the status line, ellipsised with the whole text on hover', async () => {
		setup();
		act(() => screen.getByRole('button', { name: 'Title Orange, #ff4d0d' }).focus());
		const status = screen.getByText(/Title Orange · #ff4d0d · ink C0 M70 Y95 K0/);
		expect(status).toHaveAttribute('title', 'Title Orange · #ff4d0d · ink C0 M70 Y95 K0');
		expect(status.className).toContain('ellipsis');
	});

	it('offers None for a fill that can be cleared', async () => {
		const onNone = vi.fn();
		const { props } = setup({ allowNone: true, onNone });
		await userEvent.click(screen.getByRole('button', { name: 'None, None' }));
		expect(onNone).toHaveBeenCalled();
		expect(props.onDone).toHaveBeenCalled();
	});
});

async function openCustom(user = userEvent.setup()) {
	await user.click(screen.getByRole('tab', { name: 'Custom' }));
	return user;
}

describe('Custom tab', () => {
	it('switches tabs with the arrow keys', async () => {
		const user = userEvent.setup();
		setup();
		screen.getByRole('tab', { name: 'Swatches' }).focus();
		await user.keyboard('{ArrowRight}');
		expect(screen.getByRole('tab', { name: 'Custom' })).toHaveAttribute('aria-selected', 'true');
		expect(screen.getByRole('tab', { name: 'Custom' })).toHaveFocus();
	});

	it('reads short hex without a hash and applies it on Enter', async () => {
		const { props } = setup();
		const user = await openCustom();
		const hex = screen.getByLabelText('Hex');
		await user.clear(hex);
		await user.type(hex, 'abc{Enter}');
		expect(props.onChange).toHaveBeenCalledWith('#aabbcc', true);
		expect(props.onRecord).toHaveBeenCalledWith('#aabbcc');
		expect(props.onDone).toHaveBeenCalled();
	});

	it('shows an invalid hex and applies nothing', async () => {
		const { props } = setup();
		const user = await openCustom();
		const hex = screen.getByLabelText('Hex');
		await user.clear(hex);
		await user.type(hex, '#12{Enter}');
		expect(hex).toHaveAttribute('aria-invalid', 'true');
		expect(screen.getByRole('alert')).toHaveTextContent(/3 or 6 hex digits/);
		expect(props.onChange).not.toHaveBeenCalled();
	});

	it('steps the square with the arrow keys, exposing its values, and applies on release', async () => {
		const { props } = setup({ value: '#ff0000' });
		const user = await openCustom();
		const square = screen.getByRole('slider', { name: /Saturation and brightness/ });
		expect(square).toHaveAttribute('aria-valuenow', '100');
		expect(square).toHaveAttribute(
			'aria-valuetext',
			'Saturation 100 percent, brightness 100 percent',
		);
		act(() => square.focus());
		await user.keyboard('{ArrowDown}');
		expect(square).toHaveAttribute('aria-valuenow', '99');
		expect(props.onChange).toHaveBeenCalledWith('#fc0000', false);
		await user.keyboard('{Shift>}{ArrowLeft}{/Shift}');
		expect(square).toHaveAttribute(
			'aria-valuetext',
			'Saturation 90 percent, brightness 99 percent',
		);
	});

	it('moves the hue with the keyboard', async () => {
		const { props } = setup({ value: '#ff0000' });
		await openCustom();
		const hue = screen.getByRole('slider', { name: 'Hue' });
		// A range input steps natively in a browser; here the change and the key release are sent.
		fireEvent.change(hue, { target: { value: '1' } });
		fireEvent.keyUp(hue, { key: 'ArrowRight' });
		expect(hue).toHaveValue('1');
		expect(props.onChange).toHaveBeenLastCalledWith('#ff0400', false);
	});

	it('records a refined colour once the picker goes away', async () => {
		const { props, unmount } = setup({ value: '#ff0000' });
		await openCustom();
		const hue = screen.getByRole('slider', { name: 'Hue' });
		fireEvent.change(hue, { target: { value: '1' } });
		fireEvent.keyUp(hue, { key: 'ArrowRight' });
		expect(props.onRecord).not.toHaveBeenCalled();
		unmount();
		expect(props.onRecord).toHaveBeenCalledWith('#ff0400');
	});

	it('shows RGB fields in Expert only', async () => {
		const { unmount } = setup();
		await openCustom();
		expect(screen.queryByLabelText('R')).not.toBeInTheDocument();
		unmount();
		setup({ mode: 'expert', value: '#102030' });
		await openCustom();
		expect(screen.getByLabelText('R')).toHaveValue(16);
		expect(screen.getByLabelText('G')).toHaveValue(32);
		expect(screen.getByLabelText('B')).toHaveValue(48);
	});

	it('applies typed RGB values', async () => {
		const { props } = setup({ mode: 'expert', value: '#102030' });
		const user = await openCustom();
		const red = screen.getByLabelText('R');
		await user.clear(red);
		await user.type(red, '255{Enter}');
		expect(props.onChange).toHaveBeenCalledWith('#ff2030', false);
	});

	it('judges contrast against the page for text, and leaves it out when asked not to', async () => {
		const { unmount } = setup({ value: '#000000' });
		await openCustom();
		expect(screen.getByText('21.00:1')).toBeInTheDocument();
		expect(screen.getByText(/AAA/)).toBeInTheDocument();
		unmount();
		setup({ value: '#ffffff', pageColour: '#ffffff' });
		await openCustom();
		expect(screen.getByText(/Fails WCAG/)).toBeInTheDocument();
		cleanupAll();
		setup({ contrast: undefined });
		await openCustom();
		expect(screen.queryByText(/Text colour on this page/)).not.toBeInTheDocument();
	});

	it('disables the eyedropper with the reason where the webview has none', async () => {
		setup();
		await openCustom();
		expect(screen.getByRole('button', { name: 'Eyedropper' })).toBeDisabled();
		expect(screen.getByText(/cannot sample colours/)).toBeInTheDocument();
	});

	it('applies a sampled colour where the webview has an eyedropper', async () => {
		(globalThis as { EyeDropper?: unknown }).EyeDropper = class {
			open() {
				return Promise.resolve({ sRGBHex: '#123456' });
			}
		};
		const { props } = setup();
		const user = await openCustom();
		await user.click(screen.getByRole('button', { name: 'Eyedropper' }));
		await vi.waitFor(() => expect(props.onChange).toHaveBeenCalledWith('#123456', true));
	});
});

function cleanupAll() {
	document.body.replaceChildren();
}

describe('fixed width', () => {
	const css = readFileSync(join(import.meta.dirname, 'ColourPicker.module.css'), 'utf8');

	it('takes its width from one token and never sizes to its content', () => {
		expect(css).toMatch(/\.picker\s*\{[^}]*\bwidth:\s*var\(--folio-colour-picker-width\)/);
		expect(css.replace(/\/\*[\s\S]*?\*\//g, '')).not.toMatch(/max-content|fit-content|min-content/);
		const tokens = readFileSync(join(import.meta.dirname, '..', 'tokens.css'), 'utf8');
		expect(tokens).toMatch(/--folio-colour-picker-width:\s*340px/);
	});

	it('keeps the status line and long names to one ellipsised line', () => {
		expect(css).toMatch(/\.ellipsis\s*\{[^}]*text-overflow:\s*ellipsis[^}]*white-space:\s*nowrap/);
	});

	it('is the same single element, with no inline width, whatever the tab, hover or colour', async () => {
		const longName = 'A very long swatch name that would otherwise stretch the popover sideways';
		const { container } = setup({
			swatches: [{ value: 'swatch:long', name: longName, shown: '#336699', ink: [90, 55, 35, 40] }],
		});
		const picker = container.querySelector('[data-colour-picker]') as HTMLElement;
		const states: string[] = [];
		const note = () => states.push(`${picker.getAttribute('style')}|${picker.className}`);
		note();
		screen.getByRole('button', { name: new RegExp(longName) }).focus();
		note();
		await userEvent.click(screen.getByRole('tab', { name: 'Custom' }));
		note();
		fireEvent.click(screen.getByRole('tab', { name: 'Swatches' }));
		note();
		expect(new Set(states).size).toBe(1);
		expect(picker.getAttribute('style')).toBeNull();
	});
});

describe('ColourPopover', () => {
	it('closes on Escape, returning focus to the opener when it was inside', async () => {
		const opener = document.createElement('button');
		document.body.append(opener);
		const onClose = vi.fn();
		render(
			<ColourPopover anchor={opener} label="Text colour" onClose={onClose}>
				<button type="button">inside</button>
			</ColourPopover>,
		);
		const dialog = screen.getByRole('dialog', { name: 'Text colour' });
		expect(dialog).toHaveAttribute('data-keeps-editor');
		within(dialog).getByRole('button', { name: 'inside' }).focus();
		await userEvent.keyboard('{Escape}');
		expect(onClose).toHaveBeenCalledWith('escape');
		expect(opener).toHaveFocus();
		opener.remove();
	});

	it('closes on a press away from it but not on its opener or inside it', () => {
		const opener = document.createElement('button');
		document.body.append(opener);
		const onClose = vi.fn();
		render(
			<ColourPopover anchor={opener} label="Colour" onClose={onClose}>
				<button type="button">inside</button>
			</ColourPopover>,
		);
		fireEvent.pointerDown(screen.getByRole('button', { name: 'inside' }));
		fireEvent.pointerDown(opener);
		expect(onClose).not.toHaveBeenCalled();
		fireEvent.pointerDown(document.body);
		expect(onClose).toHaveBeenCalledWith('outside');
		opener.remove();
	});
});
