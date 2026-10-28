// Tests for Jax's panel: the Ask tab
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { JaxPanel } from './JaxPanel';
import type { GuideAction } from './jaxTips';

const context = {
	picture: false,
	text: false,
	reader: 'print' as const,
	flowing: false,
	grid: false,
};

describe('JaxPanel Ask tab', () => {
	it('answers a question and offers a Show me that does what the article points to', async () => {
		const shown: GuideAction[] = [];
		render(
			<JaxPanel
				context={context}
				issues={[]}
				spreadName="Cover"
				guided={false}
				onShow={(action) => shown.push(action)}
				onHush={vi.fn()}
			/>,
		);
		await userEvent.click(screen.getByRole('tab', { name: 'Ask' }));
		await userEvent.type(screen.getByLabelText('Ask Jax a question'), 'what is bleed{enter}');
		expect(screen.getByText('What is bleed?')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Show me' }));
		expect(shown).toEqual([{ kind: 'check' }]);
	});

	it('says so when it does not know', async () => {
		render(
			<JaxPanel
				context={context}
				issues={[]}
				spreadName="Cover"
				guided={false}
				onShow={vi.fn()}
				onHush={vi.fn()}
			/>,
		);
		await userEvent.click(screen.getByRole('tab', { name: 'Ask' }));
		await userEvent.type(screen.getByLabelText('Ask Jax a question'), 'zzz{enter}');
		expect(screen.getByText(/I don't know that one yet/)).toBeInTheDocument();
	});
});
