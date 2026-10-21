// A line in the status bar for when an AI agent has just changed the book
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import { useBooksSelector } from '../books/useBooks';
import { t } from '../i18n/messages';

/** How long the line stays after the agent's last change. */
const STAYS_MS = 6000;

/** Says an agent changed the book, for a few seconds after it did. */
export function AgentChip() {
	const editedAt = useBooksSelector((s) => s.agentEditedAt);
	const [now, setNow] = useState(() => Date.now());
	useEffect(() => {
		if (editedAt === null) return;
		setNow(Date.now());
		const left = editedAt + STAYS_MS - Date.now();
		if (left <= 0) return;
		const timer = window.setTimeout(() => setNow(Date.now()), left);
		return () => window.clearTimeout(timer);
	}, [editedAt]);
	if (editedAt === null || now - editedAt >= STAYS_MS) return null;
	return <span role="status">{t('status.agent')}</span>;
}
