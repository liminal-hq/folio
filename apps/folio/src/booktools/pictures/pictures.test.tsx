// Tests for the Pictures tools: the list of pictures and the alt text
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { BooksProvider } from '../../books/BooksProvider';
import type { FileDialogs } from '../../books/dialogs';
import { createFakeBooksClient } from '../../books/fakeBooksClient';
import type { ToolProps } from '../tools';
import { AltTextTool } from './AltTextTool';
import { PicturesTool } from './PicturesTool';

const dialogs: FileDialogs = {
	pickBook: async () => null,
	pickSavePath: async () => null,
	pickImage: async () => '/pics/otter.png',
	pickText: async () => null,
};

async function mount(Tool: typeof PicturesTool, expert = false) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const apply = vi.fn(async (_intent: Intent) => {});
	const say = vi.fn();
	const props: ToolProps = {
		client: fake.client,
		id: made.summary.id,
		book: made.book,
		revision: 0,
		expert,
		apply,
		say,
	};
	render(
		<BooksProvider client={fake.client} dialogs={dialogs}>
			<Tool {...props} />
		</BooksProvider>,
	);
	return { fake, made, apply, say };
}

describe('PicturesTool', () => {
	it('lists the picture frames with how they will print, in words', async () => {
		const { fake, made } = await mount(PicturesTool);
		expect(await screen.findByText(/pictures\. /)).toBeInTheDocument();
		expect(fake.calls).toContain(`bookTool("${made.summary.id}", {"tool":"pictures"})`);
		expect(screen.queryByText(/ppi/)).toBeNull();
	});

	it('shows the pixel sizes in Expert and not in Easy', async () => {
		await mount(PicturesTool, true);
		expect(await screen.findByRole('columnheader', { name: 'Size' })).toBeInTheDocument();
	});

	it('swaps a picture for another and says so', async () => {
		const { apply, say } = await mount(PicturesTool);
		const replace = (await screen.findAllByRole('button', { name: /Replace…|Choose…/ }))[0]!;
		await userEvent.click(replace);
		await waitFor(() => expect(apply).toHaveBeenCalledTimes(1));
		expect(apply.mock.calls[0]![0]).toMatchObject({ type: 'SetImage', fit: null });
		expect(say).toHaveBeenCalled();
	});
});

describe('AltTextTool', () => {
	it('keeps what a picture shows as one change when the box is left', async () => {
		const { apply } = await mount(AltTextTool);
		const box = (await screen.findAllByRole('textbox'))[0]!;
		await userEvent.type(box, 'An otter on a log');
		expect(apply).not.toHaveBeenCalled();
		await userEvent.tab();
		expect(apply).toHaveBeenCalledTimes(1);
		expect(apply.mock.calls[0]![0]).toMatchObject({
			type: 'SetFrameDescription',
			description: 'An otter on a log',
		});
	});
});
