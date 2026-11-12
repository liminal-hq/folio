// Tests for the colour field that panels and dialogs use
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { ColourField, type ColourFieldProps } from './ColourField';

const swatches = [
	{ name: 'Cream', colour: '#f6efdc' },
	{ name: 'Orange', colour: 'swatch:orange', shown: '#ff4d0d' },
];

function setup(props: Partial<ColourFieldProps> = {}) {
	const onPick = vi.fn();
	const client = createFakeSettingsClient();
	render(
		<SettingsProvider client={client}>
			<ColourField label="Fill" value="#f6efdc" swatches={swatches} onPick={onPick} {...props} />
		</SettingsProvider>,
	);
	return { onPick, client };
}

describe('ColourField', () => {
	it('shows the name of the colour in use, or its hex', () => {
		setup();
		expect(screen.getByRole('button', { name: 'Fill' })).toHaveTextContent('Cream');
	});

	it('shows a book swatch by name and a custom colour by hex', () => {
		setup({ value: 'swatch:orange' });
		expect(screen.getByRole('button', { name: 'Fill' })).toHaveTextContent('Orange');
	});

	it('shows None for no colour', () => {
		setup({ value: null, allowNone: true });
		expect(screen.getByRole('button', { name: 'Fill' })).toHaveTextContent('None');
	});

	it('picks pure black, which the old list could not', async () => {
		const { onPick } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Fill' }));
		await userEvent.click(screen.getByRole('button', { name: 'Black, #000000' }));
		expect(onPick).toHaveBeenCalledWith('#000000');
		expect(screen.queryByRole('dialog')).toBeNull();
	});

	it('picks a typed colour from the Custom tab and remembers it as a recent colour', async () => {
		const { onPick, client } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Fill' }));
		await userEvent.click(screen.getByRole('tab', { name: 'Custom' }));
		const hex = screen.getByLabelText('Hex');
		await userEvent.clear(hex);
		await userEvent.type(hex, 'Fa0{Enter}');
		expect(onPick).toHaveBeenCalledWith('#ffaa00');
		await waitFor(async () => {
			const snapshot = await client.get();
			expect(snapshot.settings.recentColours).toEqual(['#ffaa00']);
		});
	});

	it('clears the fill only where it can be cleared', async () => {
		const { onPick } = setup({ allowNone: true });
		await userEvent.click(screen.getByRole('button', { name: 'Fill' }));
		await userEvent.click(screen.getByRole('button', { name: 'None, None' }));
		expect(onPick).toHaveBeenCalledWith(null);
	});

	it('does nothing when disabled', () => {
		setup({ disabled: true });
		expect(screen.getByRole('button', { name: 'Fill' })).toBeDisabled();
	});
});
