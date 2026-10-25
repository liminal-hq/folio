// Tests for the Connect an agent dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { AgentStatus } from '@liminal-hq/folio-protocol/generated/AgentStatus';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { AgentClient } from '../services/agentClient';
import { ConnectAgentDialog } from './ConnectAgentDialog';

const status = (extra: Partial<AgentStatus> = {}): AgentStatus => ({
	listening: true,
	endpoint: '/run/user/1000/folio/agent.sock',
	binary: '/opt/folio/folio-mcp',
	programs: [
		{ id: 'claude-code', label: 'Claude Code' },
		{ id: 'cursor', label: 'Cursor' },
	],
	...extra,
});

const open = (client: AgentClient) =>
	render(<ConnectAgentDialog open onClose={() => undefined} client={client} />);

describe('the Connect an agent dialog', () => {
	it('says whether agents can reach the open books', async () => {
		open({ status: async () => status(), install: vi.fn() });
		expect(await screen.findByText('Folio is open to agents on this computer.')).toBeVisible();
	});

	it('says so when agents cannot', async () => {
		open({ status: async () => status({ listening: false }), install: vi.fn() });
		expect(await screen.findByText(/not open to agents right now/)).toBeVisible();
	});

	it('adds Folio to a program and says what was done', async () => {
		const install = vi.fn(
			async () => 'Added Folio to ~/.cursor/mcp.json. Restart Cursor to see it.',
		);
		open({ status: async () => status(), install });
		fireEvent.click(await screen.findByRole('button', { name: 'Add Folio to Cursor' }));
		expect(await screen.findByText(/Restart Cursor to see it/)).toBeVisible();
		expect(install).toHaveBeenCalledWith('cursor');
	});

	it('says why when it could not', async () => {
		const install = vi.fn(async () => {
			throw 'no such program';
		});
		open({ status: async () => status(), install });
		fireEvent.click(await screen.findByRole('button', { name: 'Add Folio to Claude Code' }));
		expect(await screen.findByRole('alert')).toHaveTextContent('no such program');
	});

	it('offers nothing to add when the agent program is missing', async () => {
		open({ status: async () => status({ binary: null }), install: vi.fn() });
		expect(await screen.findByText(/was not found beside it/)).toBeVisible();
		expect(screen.queryByRole('button', { name: /Add Folio to/ })).toBeNull();
	});
});
