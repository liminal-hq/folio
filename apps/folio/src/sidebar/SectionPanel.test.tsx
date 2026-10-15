// Tests for starting and changing a section from the Pages group
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { SectionPanel } from './SectionPanel';

const spread = (extra: object = {}) =>
	({ id: 'sp', kind: 'spread', pages: [], frames: [], guides: [], ...extra }) as unknown as Spread;

const setup = (s: Spread, inside = '') => {
	const onApply = vi.fn();
	render(<SectionPanel spread={s} inside={inside} onApply={onApply} />);
	return onApply;
};

describe('SectionPanel', () => {
	it('is not there for the cover', () => {
		setup(spread({ kind: 'cover' }));
		expect(screen.queryByRole('heading', { name: 'Section' })).toBeNull();
	});

	it('starts a section that restarts the count, and says which section a spread is in', async () => {
		const onApply = setup(spread(), 'Front matter');
		expect(screen.getByText('In the section “Front matter”.')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Start a section here' }));
		expect(onApply).toHaveBeenCalledWith({
			type: 'SetSection',
			spread: 'sp',
			section: { name: 'New section', numbering: 'arabic', start: 1 },
		});
	});

	it('says it carries on when no section has begun', () => {
		setup(spread());
		expect(
			screen.getByText('This spread carries on the numbering from the one before.'),
		).toBeInTheDocument();
	});

	it('renames, renumbers, restarts and removes', async () => {
		const section = { name: 'Contents', numbering: 'roman', start: 3 };
		const onApply = setup(spread({ section }));
		const name = screen.getByRole('textbox', { name: 'Name' });
		await userEvent.clear(name);
		await userEvent.type(name, 'Front{Enter}');
		expect(onApply).toHaveBeenLastCalledWith({
			type: 'SetSection',
			spread: 'sp',
			section: { ...section, name: 'Front' },
		});
		await userEvent.click(screen.getByRole('radio', { name: '1, 2, 3' }));
		expect(onApply).toHaveBeenLastCalledWith({
			type: 'SetSection',
			spread: 'sp',
			section: { ...section, numbering: 'arabic' },
		});
		const start = screen.getByRole('textbox', { name: 'Start at' });
		await userEvent.clear(start);
		await userEvent.type(start, '7{Enter}');
		expect(onApply).toHaveBeenLastCalledWith({
			type: 'SetSection',
			spread: 'sp',
			section: { ...section, start: 7 },
		});
		await userEvent.click(screen.getByRole('checkbox', { name: 'Start counting again' }));
		expect(onApply).toHaveBeenLastCalledWith({
			type: 'SetSection',
			spread: 'sp',
			section: { ...section, start: null },
		});
		await userEvent.click(screen.getByRole('button', { name: 'Remove this section' }));
		expect(onApply).toHaveBeenLastCalledWith({ type: 'SetSection', spread: 'sp', section: null });
	});

	it('puts back a start that is not a whole number from 1', async () => {
		const onApply = setup(spread({ section: { name: 'a', numbering: 'arabic', start: 3 } }));
		const start = screen.getByRole('textbox', { name: 'Start at' });
		for (const bad of ['0', 'x', '2.5', '100000']) {
			await userEvent.clear(start);
			await userEvent.type(start, `${bad}{Enter}`);
		}
		expect(onApply).not.toHaveBeenCalled();
	});
});
