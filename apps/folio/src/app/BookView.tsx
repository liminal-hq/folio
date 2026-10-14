// The open book as the main window shows it: the stage and the inspector over the page shelf
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useState, type CSSProperties } from 'react';
import { shelfLabels } from '../books/shelfLabels';
import type { OpenBookCopy } from '../books/booksStore';
import { useBooks } from '../books/useBooks';
import { frameIntents } from '../canvas/frameMenu';
import { Shelf } from '../canvas/Shelf';
import { DEFAULT_MARKS, type Marks } from '../canvas/SpreadCanvas';
import { Stage } from '../canvas/Stage';
import { useSelection } from '../canvas/useSelection';
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
	const { client, store } = useBooks();
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
					spread={spread}
					layout={layout}
					selected={selected}
					marks={marks}
					onMarks={setMarks}
					apply={(intent) => void store.apply(intent)}
				/>
			</main>
			<Shelf book={book} labels={labels} selected={shown} onSelect={setSpreadIndex} />
		</>
	);
}
