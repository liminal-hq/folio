// The open book as the main window shows it: the stage and the inspector over the page shelf
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useState, type CSSProperties } from 'react';
import { shelfLabels } from '../books/shelfLabels';
import type { OpenBookCopy } from '../books/booksStore';
import { useBooks } from '../books/useBooks';
import { frameIntents } from '../canvas/frameMenu';
import { boxAt, newFrame, type NewFrameKind } from '../canvas/newFrames';
import { spreadGeometry } from '../canvas/geometry';
import { defaultTextStyle, styleChoices } from '../inspector/styleChoices';
import { Shelf } from '../canvas/Shelf';
import { shelfIntent } from '../canvas/shelfMenu';
import { DEFAULT_MARKS, type Marks } from '../canvas/SpreadCanvas';
import { Stage } from '../canvas/Stage';
import { useSelection } from '../canvas/useSelection';
import { useAssetUrls } from '../canvas/useAssetUrls';
import { useSpreadLayout } from '../canvas/useSpreadLayout';
import { Inspector } from '../inspector/Inspector';
import { useSettings } from '../settings/useSettings';
import styles from './MainScreen.module.css';

export function BookView({
	id,
	copy,
	kindLabel,
}: {
	id: string;
	copy: OpenBookCopy;
	kindLabel: string;
}) {
	const settings = useSettings();
	const { client, store, dialogs } = useBooks();
	const [spreadIndex, setSpreadIndex] = useState(0);
	const [marks, setMarks] = useState<Marks>(DEFAULT_MARKS);
	const book = copy.book;
	const shown = Math.max(0, Math.min(spreadIndex, book.spreads.length - 1));
	const spread = book.spreads[shown] ?? null;
	const labels = shelfLabels(book);
	const layout = useSpreadLayout(client, id, spread ? spread.id : null, copy.revision);
	const [selection, select] = useSelection(
		spread?.id ?? null,
		spread?.frames.map((f) => f.id) ?? [],
	);
	const images = useAssetUrls(
		client,
		id,
		(spread?.frames ?? []).flatMap((f) =>
			f.type === 'image' && f.image.asset ? [f.image.asset] : [],
		),
	);
	if (!spread) return null;
	const selected = spread.frames.filter((f) => selection.includes(f.id));

	return (
		<>
			<main
				className={styles.body}
				style={
					{ '--folio-inspector-width': `${settings.interface.sidebarWidth}px` } as CSSProperties
				}
			>
				<div className={styles.stage}>
					<Stage
						book={book}
						spread={spread}
						label={labels[shown] ?? ''}
						layout={layout}
						marks={marks}
						images={images}
						selection={selection}
						onSelect={select}
						onMove={(frame, box) =>
							store.apply({
								type: 'MoveFrame',
								spread: spread.id,
								frame,
								box: { x: box.x, y: box.y, w: box.w, h: box.h },
							})
						}
						onEditText={(frame, text) => void store.apply({ type: 'SetText', frame, text })}
						onPageCommand={(command, point) => {
							const kinds: Record<string, NewFrameKind> = {
								addText: 'text',
								addPicture: 'picture',
								addNote: 'note',
							};
							const kind = kinds[command];
							if (kind) {
								const pages = spreadGeometry(book, spread).pages;
								const page =
									pages.find((p) => point.x >= p.trim.x && point.x <= p.trim.x + p.trim.w) ??
									pages[0]!;
								const choices = styleChoices(copy.styles);
								const style = defaultTextStyle(choices);
								const frame = newFrame(kind, boxAt(kind, point, page.trim), style);
								void store.apply({ type: 'AddFrame', spread: spread.id, frame });
								select([frame.id]);
							} else if (command === 'addPages') {
								void store.apply({ type: 'AddSpread', after: spread.id, duplicate: false });
								setSpreadIndex(shown + 1);
							} else if (command === 'duplicateSpread') {
								void store.apply({ type: 'AddSpread', after: spread.id, duplicate: true });
								setSpreadIndex(shown + 1);
							} else if (command === 'toggleMargins') {
								setMarks({ ...marks, margins: !marks.margins });
							} else if (command === 'toggleGrid') {
								const on = !(marks.columns || marks.baseline);
								setMarks({ ...marks, columns: on, baseline: on });
							}
						}}
						onCommand={(command, ids) => {
							for (const frameId of ids) {
								const target = spread.frames.find((f) => f.id === frameId);
								if (!target) continue;
								for (const intent of frameIntents(command, spread.id, target)) {
									void store.apply(intent);
								}
							}
						}}
					/>
				</div>
				<Inspector
					book={book}
					styleFile={copy.styles}
					kindLabel={kindLabel}
					units={settings.units}
					mode={settings.mode}
					spread={spread}
					layout={layout}
					selected={selected}
					marks={marks}
					onMarks={setMarks}
					apply={(intent) => void store.apply(intent)}
					choosePicture={async (frame) => {
						const picked = await dialogs.pickImage();
						if (picked === null) return;
						try {
							const asset = await client.importImage(id, picked);
							await store.apply({ type: 'SetImage', frame, asset, fit: null });
						} catch {
							// The store keeps the refusal for the page to say.
						}
					}}
				/>
			</main>
			<Shelf
				book={book}
				labels={labels}
				selected={shown}
				onSelect={setSpreadIndex}
				onAdd={() => {
					// New pages go at the end of the book.
					const last = book.spreads[book.spreads.length - 1]!;
					void store.apply({ type: 'AddSpread', after: last.id, duplicate: false });
					setSpreadIndex(book.spreads.length);
				}}
				onCommand={(command, index) => {
					const intent = shelfIntent(command, book, index);
					if (!intent) return;
					void store.apply(intent);
					if (command === 'duplicate' || command === 'moveRight') setSpreadIndex(index + 1);
					else if (command === 'moveLeft') setSpreadIndex(index - 1);
					else if (command === 'remove') setSpreadIndex(Math.max(0, index - 1));
				}}
				onMove={(from, to) => {
					const moved = book.spreads[from];
					if (!moved) return;
					void store.apply({ type: 'MoveSpread', spread: moved.id, to });
					setSpreadIndex(Math.max(1, to));
				}}
			/>
		</>
	);
}
