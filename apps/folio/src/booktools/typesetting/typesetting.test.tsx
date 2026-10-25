// Tests for the Typesetting tools: smart punctuation and consistent styles
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
import { PunctuationTool } from './PunctuationTool';
import { StylesTool } from './StylesTool';

async function mount(Tool: ComponentType<ToolProps>) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const say = vi.fn();
	render(
		<Tool
			client={fake.client}
			id={made.summary.id}
			book={made.book}
			revision={0}
			expert={false}
			apply={vi.fn(async (_intent: Intent) => {})}
			say={say}
		/>,
	);
	return { fake, say };
}

describe('PunctuationTool', () => {
	it('shows each rule with how many places and what it would look like, and tidies the ticked ones', async () => {
		const { fake, say } = await mount(PunctuationTool);
		expect(await screen.findByText('Curly quotes and apostrophes')).toBeInTheDocument();
		expect(screen.getByText('4 places')).toBeInTheDocument();
		expect(screen.getByText('said “hello” and')).toBeInTheDocument();
		// Leave the dashes alone and tidy only the quotes.
		await userEvent.click(screen.getByRole('checkbox', { name: /Dashes/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Tidy 4 places' }));
		expect(fake.calls.at(-1)).toContain('"rules":["quotes"]');
		expect(say).toHaveBeenCalledWith('Tidied 5 paragraphs. One undo takes it all back.');
	});
});

describe('StylesTool', () => {
	it('lists the text boxes that differ from their style and puts them back', async () => {
		const { fake, say } = await mount(StylesTool);
		expect(await screen.findByText(/body style, with its own size, weight/)).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Put all 1 back to their style' }));
		expect(fake.calls.at(-1)).toContain('"tool":"resetStyles"');
		expect(say).toHaveBeenCalledWith('Put 1 text boxes back to their style.');
	});
});
