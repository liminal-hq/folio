// Tests for the Structure tools: the copyright page and the contents
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ComponentType } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../../books/fakeBooksClient';
import type { ToolProps } from '../tools';
import { ContentsTool } from './ContentsTool';
import { CopyrightTool } from './CopyrightTool';

async function mount(Tool: ComponentType<ToolProps>) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const say = vi.fn();
	render(
		<Tool
			client={fake.client}
			id={made.summary.id}
			book={{ ...made.book, authors: [{ name: 'Jane L. Morris', role: 'author' }] }}
			revision={0}
			expert={false}
			apply={vi.fn(async (_intent: Intent) => {})}
			say={say}
		/>,
	);
	return { fake, say };
}

describe('CopyrightTool', () => {
	it('starts from the author, shows the words the page would say, and adds the page', async () => {
		const { fake, say } = await mount(CopyrightTool);
		expect(await screen.findByLabelText('The copyright page')).toHaveTextContent(
			/Copyright © \d{4} Jane L\. Morris/,
		);
		await userEvent.click(screen.getByRole('button', { name: 'Add the copyright page' }));
		expect(fake.calls.at(-1)).toContain('"tool":"copyrightPage"');
		expect(say).toHaveBeenCalledWith('Added the copyright page after the title page.');
	});

	it('asks for the new words again when a field changes', async () => {
		const { fake } = await mount(CopyrightTool);
		const holder = await screen.findByLabelText('Who holds the copyright');
		await userEvent.clear(holder);
		await userEvent.type(holder, 'Mom');
		expect(fake.calls.some((c) => c.includes('"holder":"Mom"'))).toBe(true);
	});
});

describe('ContentsTool', () => {
	it('lists the chapters with their pages and adds the contents page', async () => {
		const { fake, say } = await mount(ContentsTool);
		expect(await screen.findByText('The pumpkin')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Add the contents page' }));
		expect(fake.calls.at(-1)).toContain('"tool":"contentsPage"');
		expect(say).toHaveBeenCalledWith('Added the contents page.');
	});
});
