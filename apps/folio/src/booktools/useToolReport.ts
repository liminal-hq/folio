// Asks Rust for a tool's report, and again each time the book changes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ToolReport } from '@liminal-hq/folio-protocol/generated/ToolReport';
import type { ToolRequest } from '@liminal-hq/folio-protocol/generated/ToolRequest';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';

/** The latest report, kept on show while the next is asked for; `null` before the first arrives. */
export function useToolReport<T extends ToolReport['tool']>(
	client: BooksClient,
	id: string,
	request: Extract<ToolRequest, { tool: T }>,
	revision: number,
): Extract<ToolReport, { tool: T }> | null {
	const [held, setHeld] = useState<{ id: string; report: ToolReport } | null>(null);
	const key = JSON.stringify(request);
	useEffect(() => {
		let live = true;
		client
			.bookTool(id, request)
			.then((report) => live && setHeld({ id, report }))
			.catch((error: unknown) => console.warn('A Book tool could not answer:', error));
		return () => {
			live = false;
		};
		// The request is its key, so a new object for the same question is not asked again.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [client, id, key, revision]);
	return held && held.id === id && held.report.tool === request.tool
		? (held.report as Extract<ToolReport, { tool: T }>)
		: null;
}
