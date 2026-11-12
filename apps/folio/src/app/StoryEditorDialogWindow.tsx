// The story editor as a window of its own: the whole text of a story, with spelling and styles
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useMemo } from 'react';
import { useBooks } from '../books/useBooks';
import type { DialogPayloads } from '../dialogs/dialogKinds';
import type { DialogSession } from '../dialogs/DialogHost';
import { useDialogBook } from '../dialogs/useDialogBook';
import { styleChoices } from '../inspector/styleChoices';
import { StoryEditorDialog } from '../story/StoryEditorDialog';
import { useSpeller } from './useSpeller';

export function StoryEditorDialogWindow({
	payload,
	session,
}: {
	payload: DialogPayloads['StoryEditor'];
	session: DialogSession;
}) {
	const dialog = useDialogBook(session);
	const { client } = useBooks();
	const speller = useSpeller(client, payload.bookId, (intent) => dialog?.apply(intent));
	const choices = useMemo(() => styleChoices(dialog?.styles), [dialog?.styles]);
	if (!dialog) return null;
	return (
		<StoryEditorDialog
			open
			onClose={dialog.close}
			book={dialog.book}
			flow={payload.flow}
			focusParagraph={payload.paragraph}
			speller={speller}
			choices={choices}
			apply={dialog.apply}
		/>
	);
}
