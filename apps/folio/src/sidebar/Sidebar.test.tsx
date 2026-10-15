// Tests for the sidebar: its tabs and the panel of the open group
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { Sidebar, type SidebarGroup } from './Sidebar';

function Harness({ start = 'properties' as SidebarGroup }) {
	const [group, setGroup] = useState<SidebarGroup>(start);
	return (
		<Sidebar
			group={group}
			onGroup={setGroup}
			panels={{
				properties: <p>The properties</p>,
				pages: <p>The pages</p>,
				proofing: <p>The proofing</p>,
			}}
		/>
	);
}

describe('Sidebar', () => {
	it('shows the open group and no other, labelled by its tab', () => {
		render(<Harness />);
		expect(screen.getByRole('tab', { name: 'Properties' })).toHaveAttribute(
			'aria-selected',
			'true',
		);
		expect(screen.getByRole('tabpanel', { name: 'Properties' })).toHaveTextContent(
			'The properties',
		);
		expect(screen.queryByText('The pages')).toBeNull();
	});

	it('opens another group by its tab', async () => {
		render(<Harness />);
		await userEvent.click(screen.getByRole('tab', { name: 'Pages' }));
		expect(screen.getByRole('tabpanel', { name: 'Pages' })).toHaveTextContent('The pages');
		expect(screen.queryByText('The properties')).toBeNull();
	});

	it('moves between the tabs with the arrow keys, Home and End, and only the open tab is a tab stop', async () => {
		render(<Harness />);
		const properties = screen.getByRole('tab', { name: 'Properties' });
		const pages = screen.getByRole('tab', { name: 'Pages' });
		expect(properties).toHaveAttribute('tabindex', '0');
		expect(pages).toHaveAttribute('tabindex', '-1');
		properties.focus();
		const proofing = screen.getByRole('tab', { name: 'Proofing' });
		await userEvent.keyboard('{ArrowRight}');
		expect(pages).toHaveFocus();
		expect(pages).toHaveAttribute('aria-selected', 'true');
		await userEvent.keyboard('{ArrowRight}');
		expect(proofing).toHaveFocus();
		await userEvent.keyboard('{ArrowRight}');
		expect(properties).toHaveFocus();
		await userEvent.keyboard('{End}');
		expect(proofing).toHaveFocus();
		await userEvent.keyboard('{Home}');
		expect(properties).toHaveFocus();
	});
});
