// The story editor: a flow's paragraphs in a list, edited a paragraph at a time
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { useEffect, useRef, useState } from 'react';
import { charsBefore } from '../canvas/textSelection';
import { t } from '../i18n/messages';
import type { StyleChoice } from '../inspector/styleChoices';
import styles from './StoryEditorDialog.module.css';

interface StoryEditorDialogProps {
	open: boolean;
	onClose(): void;
	book: Book;
	/** The flow being edited. */
	flow: string | null;
	/** The paragraph to put the caret in when the editor opens. */
	focusParagraph?: number | null;
	choices: StyleChoice[];
	apply(intent: Intent): void;
}

/** Where the caret goes once the story has changed: a paragraph and a character in it. */
interface Caret {
	para: number;
	at: number;
}

/** The UTF-16 position of character `chars` of `text`. */
const unitsOf = (text: string, chars: number) => [...text].slice(0, chars).join('').length;

interface ParagraphFieldProps {
	index: number;
	text: string;
	style: string | undefined;
	choices: StyleChoice[];
	caret: Caret | null;
	onCaret(): void;
	onText(text: string): void;
	onStyle(style: string | null): void;
	onSplit(at: number): void;
	onJoin(): void;
}

function ParagraphField({
	index,
	text,
	style,
	choices,
	caret,
	onCaret,
	onText,
	onStyle,
	onSplit,
	onJoin,
}: ParagraphFieldProps) {
	const area = useRef<HTMLTextAreaElement | null>(null);
	const [value, setValue] = useState(text);
	/** What the book was last told, so an undo (a different text) is told apart from typing. */
	const sent = useRef(text);
	useEffect(() => {
		if (text !== sent.current) {
			sent.current = text;
			setValue(text);
		}
	}, [text]);
	// The box grows with its words.
	useEffect(() => {
		const el = area.current;
		if (!el) return;
		el.style.height = 'auto';
		el.style.height = `${el.scrollHeight}px`;
	}, [value]);
	useEffect(() => {
		if (caret?.para !== index) return;
		const el = area.current;
		if (!el) return;
		el.focus();
		const at = unitsOf(el.value, caret.at);
		el.setSelectionRange(at, at);
		onCaret();
	}, [caret, index, onCaret]);

	return (
		<div className={styles.paragraph}>
			<select
				className={styles.style}
				aria-label={t('story.paragraphStyle', { n: index + 1 })}
				value={style ?? ''}
				onChange={(e) => onStyle(e.target.value || null)}
			>
				<option value="">{t('story.frameStyle')}</option>
				{choices.map((c) => (
					<option key={c.id} value={c.id}>
						{c.name}
					</option>
				))}
			</select>
			<textarea
				ref={area}
				className={styles.words}
				aria-label={t('story.paragraph', { n: index + 1 })}
				rows={1}
				value={value}
				onChange={(e) => {
					setValue(e.target.value);
					sent.current = e.target.value;
					onText(e.target.value);
				}}
				onKeyDown={(e) => {
					const el = e.currentTarget;
					if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey && !e.metaKey) {
						e.preventDefault();
						onSplit(charsBefore(el.value, el.selectionStart));
					} else if (
						e.key === 'Backspace' &&
						el.selectionStart === 0 &&
						el.selectionEnd === 0 &&
						index > 0
					) {
						e.preventDefault();
						onJoin();
					}
				}}
			/>
		</div>
	);
}

export function StoryEditorDialog({
	open,
	onClose,
	book,
	flow,
	focusParagraph = null,
	choices,
	apply,
}: StoryEditorDialogProps) {
	const story = book.flows.find((f) => f.id === flow);
	const [caret, setCaret] = useState<Caret | null>(null);
	const paragraphs = story?.paragraphs ?? [];
	const clear = useRef(() => setCaret(null)).current;
	useEffect(() => {
		if (open && focusParagraph !== null) setCaret({ para: focusParagraph, at: 0 });
	}, [open, focusParagraph]);

	return (
		<Dialog
			open={open && story !== undefined}
			title={t('story.title')}
			size="large"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton variant="primary" onClick={onClose}>
						{t('story.done')}
					</DialogButton>
				</DialogActions>
			}
		>
			<p className={styles.hint}>{t('story.hint')}</p>
			<div className={styles.list}>
				{paragraphs.map((para, i) => (
					<ParagraphField
						key={i}
						index={i}
						text={para.inline.map((r) => r.text).join('')}
						style={para.style ?? undefined}
						choices={choices}
						caret={caret}
						onCaret={clear}
						onText={(text) => apply({ type: 'SetParagraphText', flow: story!.id, para: i, text })}
						onStyle={(style) =>
							apply({
								type: 'StyleParagraphs',
								flow: story!.id,
								from: i,
								to: i,
								style,
							})
						}
						onSplit={(at) => {
							apply({ type: 'SplitParagraph', flow: story!.id, para: i, at });
							setCaret({ para: i + 1, at: 0 });
						}}
						onJoin={() => {
							const before = [...(paragraphs[i - 1]?.inline.map((r) => r.text).join('') ?? '')]
								.length;
							apply({ type: 'JoinParagraphs', flow: story!.id, para: i - 1 });
							setCaret({ para: i - 1, at: before });
						}}
					/>
				))}
			</div>
		</Dialog>
	);
}
