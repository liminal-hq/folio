// Asks the app whether agents can reach its books, and adds Folio to an agent program
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import type { AgentStatus } from '@liminal-hq/folio-protocol/generated/AgentStatus';

/** What the Connect an agent dialog needs from the app. */
export interface AgentClient {
	status(): Promise<AgentStatus>;
	/** Adds Folio to an agent program's list of MCP servers; says what was done, or rejects with why not. */
	install(program: string): Promise<string>;
}

/** The Rust `agent_status` and `agent_install` commands, which the app itself registers. */
export const tauriAgentClient: AgentClient = {
	status: () => invoke<AgentStatus>('agent_status'),
	install: (program) => invoke<string>('agent_install', { program }),
};
