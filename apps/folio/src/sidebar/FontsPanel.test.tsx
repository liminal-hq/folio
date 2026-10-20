// Tests for the Fonts panel: what the book uses and the choice to keep a font with it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FontUse } from '@liminal-hq/folio-protocol/generated/FontUse';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { FontsPanel } from './FontsPanel';

const uses: FontUse[] = [
	{ family: 'Literata', origin: 'bundled', missing: false, licence: 'free' },
	{ family: 'Zed Sans', origin: 'system', missing: false, licence: 'free' },
	{ family: 'Quill Mono', origin: 'book', missing: false, licence: 'free' },
	{ family: 'Gill', origin: 'system', missing: false, licence: 'printOnly' },
	{ family: 'Lost Serif', origin: 'system', missing: true, licence: 'restricted' },
];

const row = (family: string) => screen.getByText(family).closest('li')!;

describe('the Fonts panel', () => {
	it('says it is looking until the plugin answers', () => {
		render(<FontsPanel uses={undefined} onKeep={() => {}} onLetGo={() => {}} />);
		expect(screen.getByText('Looking at the fonts…')).toBeInTheDocument();
	});

	it('says where each font is from, and flags one that is missing or may not be kept', () => {
		render(<FontsPanel uses={uses} onKeep={() => {}} onLetGo={() => {}} />);
		expect(within(row('Literata')).getByText('Comes with Folio')).toBeInTheDocument();
		expect(within(row('Zed Sans')).getByText('On this computer')).toBeInTheDocument();
		expect(within(row('Quill Mono')).getByText('Kept in this book')).toBeInTheDocument();
		expect(
			within(row('Lost Serif')).getByText(/Not on this computer or in the book/),
		).toBeInTheDocument();
		expect(within(row('Gill')).getByText(/printed into a PDF but not kept/)).toBeInTheDocument();
	});

	it('offers to keep an installed font that may be kept, and to let go of one that is kept', async () => {
		const onKeep = vi.fn();
		const onLetGo = vi.fn();
		render(<FontsPanel uses={uses} onKeep={onKeep} onLetGo={onLetGo} />);
		await userEvent.click(
			within(row('Zed Sans')).getByRole('button', { name: 'Keep in the book' }),
		);
		expect(onKeep).toHaveBeenCalledWith('Zed Sans');
		await userEvent.click(within(row('Quill Mono')).getByRole('button', { name: 'Stop keeping' }));
		expect(onLetGo).toHaveBeenCalledWith('Quill Mono');
		for (const family of ['Literata', 'Gill', 'Lost Serif']) {
			expect(within(row(family)).queryByRole('button')).toBeNull();
		}
	});
});
