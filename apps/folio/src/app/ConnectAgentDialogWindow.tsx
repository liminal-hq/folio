// The Connect an agent dialog as a window of its own
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DialogSession } from '../dialogs/DialogHost';
import { tauriAgentClient, type AgentClient } from '../services/agentClient';
import { ConnectAgentDialog } from './ConnectAgentDialog';

export function ConnectAgentDialogWindow({
	session,
	client = tauriAgentClient,
}: {
	session: DialogSession;
	client?: AgentClient;
}) {
	return <ConnectAgentDialog open onClose={session.close} client={client} />;
}
