// The open book as the main window shows it: the stage and the inspector over the page shelf
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RibbonCommand } from '@liminal-hq/folio-ribbon/types';
import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react';
import { shelfLabels } from '../books/shelfLabels';
import type { OpenBookCopy } from '../books/booksStore';
import { useBooks } from '../books/useBooks';
import { frameIntents } from '../canvas/frameMenu';
import { boxAt, newFrame, type NewFrameKind } from '../canvas/newFrames';
import { spreadGeometry } from '../canvas/geometry';
import { masterFurniture } from '../canvas/masterItems';
import { masterOfPage, masterSpread, surfaceOf } from '../canvas/masterView';
import { defaultTextStyle, styleChoices } from '../inspector/styleChoices';
import { Shelf } from '../canvas/Shelf';
import { shelfIntent } from '../canvas/shelfMenu';
import { guideIntents } from '../canvas/guides';
import { coloursOf, marksOf, marksPatch, type MarkKey, type Marks } from '../canvas/marksModel';
import { Stage, type ZoomApi } from '../canvas/Stage';
import { useSelection } from '../canvas/useSelection';
import { useAssetUrls } from '../canvas/useAssetUrls';
import { useSpreadLayout } from '../canvas/useSpreadLayout';
import type { CommandId } from '../commands/appMenuModel';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { Inspector } from '../inspector/Inspector';
import { t } from '../i18n/messages';
import {
	editorAction,
	editorState,
	type EditorAction,
	type EditorInput,
} from '../ribbon/editorCommands';
import { FolioToolbar } from '../ribbon/FolioToolbar';
import type { SettingsPatch } from '@liminal-hq/folio-plugin-settings';
import { useSettings, useUpdateSettings } from '../settings/useSettings';
import styles from './MainScreen.module.css';

/** A box with only the four numbers the book keeps. */
const boxOf = (r: { x: number; y: number; w: number; h: number }) => ({
	x: r.x,
	y: r.y,
	w: r.w,
	h: r.h,
});

export function BookView({
	id,
	copy,
	kindLabel,
	available,
	run,
}: {
	id: string;
	copy: OpenBookCopy;
	kindLabel: string;
	/** The menu commands that can be run now, for the toolbar. */
	available: ReadonlySet<CommandId>;
	run: (id: string) => Promise<void>;
}) {
	const settings = useSettings();
	const { client, store, dialogs } = useBooks();
	const [spreadIndex, setSpreadIndex] = useState(0);
	/** The master page being edited, in place of the shelf's spread, or `null`. */
	const [editing, setEditing] = useState<string | null>(null);
	const update = useUpdateSettings();
	const marks = useMemo(() => marksOf(settings.marks), [settings.marks]);
	const colours = useMemo(() => coloursOf(settings.marks), [settings.marks]);
	const snap = settings.marks.snap;
	const rulers = settings.rulers;
	/** Changes which marks are drawn; the choice is kept in the settings, for every book. */
	const setMarks = (next: Partial<Marks>) => void update(marksPatch(marks, next));
	const zoomApi = useRef<ZoomApi | null>(null);
	const book = copy.book;
	const shown = Math.max(0, Math.min(spreadIndex, book.spreads.length - 1));
	const master = editing === null ? null : masterSpread(book, editing);
	const spread = master ?? book.spreads[shown] ?? null;
	const labels = shelfLabels(book);
	const layout = useSpreadLayout(
		client,
		id,
		master ? editing : (spread?.id ?? null),
		copy.revision,
		master !== null,
	);
	const [selection, select] = useSelection(
		spread?.id ?? null,
		spread?.frames.map((f) => f.id) ?? [],
	);
	const images = useAssetUrls(
		client,
		id,
		[...(spread?.frames ?? []), ...(spread ? masterFurniture(book, spread) : [])].flatMap((f) =>
			f.type === 'image' && f.image.asset ? [f.image.asset] : [],
		),
	);
	const choices = useMemo(() => styleChoices(copy.styles), [copy.styles]);
	const selected = useMemo(
		() => spread?.frames.filter((f) => selection.includes(f.id)) ?? [],
		[spread, selection],
	);
	const input: EditorInput | null = spread
		? {
				spread,
				spreadIndex: shown,
				spreadCount: book.spreads.length,
				selected,
				layout,
				choices,
				marks,
				snap,
				rulers: rulers.on,
				master: master !== null,
			}
		: null;
	const editor = useMemo(
		() => (input ? editorState(input) : undefined),
		[spread, shown, book.spreads.length, selected, layout, choices, marks, snap, rulers.on],
	);

	// Ctrl+R shows and hides the rulers, and is kept from reloading the page.
	useEffect(() => {
		const onKey = (event: KeyboardEvent) => {
			if (!(event.ctrlKey || event.metaKey) || event.altKey || event.shiftKey) return;
			if (event.key.toLowerCase() !== 'r') return;
			event.preventDefault();
			void update({ rulers: { on: !rulers.on } } as SettingsPatch);
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	}, [rulers.on, update]);

	// A master that is gone (undone, or removed with the page numbers) ends the editing of it.
	useEffect(() => {
		if (editing !== null && masterSpread(book, editing) === null) setEditing(null);
	}, [book, editing]);

	// Escape leaves the master page, unless it is ending the editing of some words.
	useEffect(() => {
		if (editing === null) return;
		const onKey = (event: KeyboardEvent) => {
			if (event.key !== 'Escape' || event.defaultPrevented) return;
			if (
				event.target instanceof HTMLElement &&
				/^(INPUT|TEXTAREA|SELECT)$/.test(event.target.tagName)
			)
				return;
			setEditing(null);
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	}, [editing]);
	if (!spread || !input) return null;
	const surface = surfaceOf(spread);
	const shownSpread = book.spreads[shown];
	const demoteTo =
		master && shownSpread?.pages.some((p) => p.master === editing) ? shownSpread.id : null;
	const masterName = book.masterPages?.find((m) => m.id === editing)?.name ?? '';

	const apply = (intent: Intent) => void store.apply(intent);

	/** Puts a new frame of `kind` on the spread, centred on `point` (in points) or the first page. */
	const addFrame = (kind: NewFrameKind, point?: { x: number; y: number }) => {
		const pages = spreadGeometry(book, spread).pages;
		const first = pages[0]!;
		const at = point ?? {
			x: first.trim.x + first.trim.w / 2,
			y: first.trim.y + first.trim.h / 2,
		};
		const page = pages.find((p) => at.x >= p.trim.x && at.x <= p.trim.x + p.trim.w) ?? first;
		const frame = newFrame(kind, boxAt(kind, at, page.trim), defaultTextStyle(choices));
		apply({ type: 'AddFrame', on: surface, frame });
		select([frame.id]);
	};

	const pageOp = (op: 'add' | 'duplicate' | 'remove', index = shown) => {
		const target = book.spreads[index];
		if (!target) return;
		if (op === 'add') {
			// New pages go at the end of the book.
			const last = book.spreads[book.spreads.length - 1]!;
			apply({ type: 'AddSpread', after: last.id, duplicate: false });
			setSpreadIndex(book.spreads.length);
		} else if (op === 'duplicate' && target.kind !== 'cover') {
			apply({ type: 'AddSpread', after: target.id, duplicate: true });
			setSpreadIndex(index + 1);
		} else if (op === 'remove' && target.kind !== 'cover' && book.spreads.length > 1) {
			apply({ type: 'DeleteSpread', spread: target.id });
			setSpreadIndex(Math.max(0, index - 1));
		}
	};

	const choosePicture = async (frame: string) => {
		const picked = await dialogs.pickImage();
		if (picked === null) return;
		try {
			const asset = await client.importImage(id, picked);
			// The book on show may have changed while the picker was open.
			if (store.state.getState().activeId !== id) return;
			await store.apply({ type: 'SetImage', frame, asset, fit: null });
		} catch {
			// The store keeps the refusal for the page to say.
		}
	};

	const stretchToBleed = (frame: Frame) => {
		const geometry = spreadGeometry(book, spread);
		const centre = frame.box.x + frame.box.w / 2;
		const page =
			geometry.pages.find((p) => centre >= p.trim.x && centre <= p.trim.x + p.trim.w) ??
			geometry.pages[0]!;
		const first = page === geometry.pages[0];
		const last = page === geometry.pages[geometry.pages.length - 1];
		const x = page.trim.x - (first ? book.bleed : 0);
		const right = page.trim.x + page.trim.w + (last ? book.bleed : 0);
		apply({
			type: 'MoveFrame',
			on: surface,
			frame: frame.id,
			box: { x, y: -book.bleed, w: right - x, h: page.trim.h + book.bleed * 2 },
		});
	};

	/**
	 * Opens the master the page at `index` uses for editing, making the book's masters first when it
	 * has none (`index` is -1 when no page has one).
	 */
	const editMasterOf = async (index: number) => {
		const named = index === -1 ? null : masterOfPage(spread, index);
		if (named === null) {
			await store.apply({ type: 'EnsureMasters' });
			// The store keeps a refusal for the page to say, and there is no master to edit then.
			if (store.state.getState().error === null) setEditing('mst_a');
		} else setEditing(named);
	};

	const perform = (action: EditorAction) => {
		switch (action.kind) {
			case 'intents':
				action.intents.forEach(apply);
				break;
			case 'add':
				addFrame(action.shape);
				break;
			case 'marks':
				setMarks(action.marks);
				break;
			case 'snap':
				void update({ marks: { snap: !snap } } as SettingsPatch);
				break;
			case 'rulers':
				void update({ rulers: { on: !rulers.on } } as SettingsPatch);
				break;
			case 'zoom':
				if (action.zoom === 'in') zoomApi.current?.zoomIn();
				else if (action.zoom === 'out') zoomApi.current?.zoomOut();
				else zoomApi.current?.fit();
				break;
			case 'page':
				pageOp(action.op);
				break;
			case 'choosePicture':
				void choosePicture(action.frame);
				break;
			case 'stretch': {
				const frame = spread.frames.find((f) => f.id === action.frame);
				if (frame) stretchToBleed(frame);
				break;
			}
			case 'fit':
				apply({ type: 'SetImage', frame: action.frame, asset: null, fit: action.fit });
				break;
			case 'editMaster': {
				if (editing !== null) setEditing(null);
				else void editMasterOf(spread.pages.findIndex((p) => p.master));
				break;
			}
		}
	};

	/** The ribbon's document controls; answers whether the command was one of them. */
	const onEditorCommand = (command: RibbonCommand): boolean => {
		const value = typeof command.value === 'boolean' ? undefined : command.value;
		const action = editorAction(command.id, value, input);
		if (!action) return false;
		perform(action);
		return true;
	};

	return (
		<>
			<FolioToolbar
				available={available}
				run={run}
				editor={editor}
				contextual={editor?.contextual}
				onEditorCommand={onEditorCommand}
			/>
			<main
				className={styles.body}
				style={
					{ '--folio-inspector-width': `${settings.interface.sidebarWidth}px` } as CSSProperties
				}
			>
				<div className={styles.stage}>
					{master && (
						<div className={styles.masterBanner} role="status">
							<span>{t('master.editing', { name: masterName })}</span>
							<span className={styles.masterHint}>{t('master.editing.hint')}</span>
							<button type="button" onClick={() => setEditing(null)}>
								{t('master.done')}
							</button>
						</div>
					)}
					<Stage
						book={book}
						spread={spread}
						label={master ? t('master.page', { name: masterName }) : (labels[shown] ?? '')}
						heading={master ? t('master.editing', { name: masterName }) : undefined}
						layout={layout}
						marks={marks}
						colours={colours}
						onMarks={setMarks}
						onMarkColour={(key: MarkKey, colour: number) =>
							void update({ marks: { [key]: { colour } } } as SettingsPatch)
						}
						snap={snap}
						rulers={{ on: rulers.on, unit: rulers.units }}
						onRulerUnit={(units) => void update({ rulers: { units } } as SettingsPatch)}
						onAddGuide={(axis, pos) => apply({ type: 'AddGuide', on: surface, axis, pos })}
						onMoveGuide={(index, pos) => apply({ type: 'MoveGuide', on: surface, index, pos })}
						onRemoveGuide={(index) => apply({ type: 'RemoveGuide', on: surface, index })}
						onGuideCommand={(command, index) => {
							for (const intent of guideIntents(command, spread, index, demoteTo)) apply(intent);
						}}
						zoomApi={zoomApi}
						images={images}
						selection={selection}
						onSelect={select}
						onMove={(moves) =>
							store.apply(
								moves.length === 1
									? {
											type: 'MoveFrame',
											on: surface,
											frame: moves[0]!.id,
											box: boxOf(moves[0]!.rect),
										}
									: {
											type: 'MoveFrames',
											on: surface,
											moves: moves.map((m) => ({ frame: m.id, box: boxOf(m.rect) })),
										},
							)
						}
						onEditText={(frame, text) => void store.apply({ type: 'SetText', frame, text })}
						editableMasters={settings.mode === 'expert'}
						demoteTo={demoteTo}
						onPageCommand={(command, point) => {
							const kinds: Record<string, NewFrameKind> = {
								addText: 'text',
								addPicture: 'picture',
								addNote: 'note',
							};
							const kind = kinds[command];
							if (kind) addFrame(kind, point);
							else if (command.startsWith('editMaster:'))
								setEditing(command.slice('editMaster:'.length));
							else if (command === 'toggleMargins') setMarks({ margins: !marks.margins });
							else if (command === 'toggleGrid') {
								const on = !(marks.columns || marks.baseline);
								setMarks({ columns: on, baseline: on });
							} else if (master) return;
							else if (command === 'addPages') {
								apply({ type: 'AddSpread', after: spread.id, duplicate: false });
								setSpreadIndex(shown + 1);
							} else if (command === 'duplicateSpread') pageOp('duplicate');
							else if (command.startsWith('copyMaster:')) {
								const [, page, frame] = command.split(':') as [string, string, string];
								apply({ type: 'CopyMasterItem', spread: spread.id, page, frame });
							} else if (command.startsWith('hideMaster:') || command.startsWith('showMaster:')) {
								const [which, page, frame] = command.split(':') as [string, string, string];
								apply({
									type: 'HideMasterItem',
									spread: spread.id,
									page,
									frame,
									hide: which === 'hideMaster',
								});
							}
						}}
						onCommand={(command, ids) => {
							for (const frameId of ids) {
								const target = spread.frames.find((f) => f.id === frameId);
								if (!target) continue;
								for (const intent of frameIntents(command, surface, target)) apply(intent);
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
					onMarks={(next) => setMarks(next)}
					apply={apply}
					choosePicture={(frame) => void choosePicture(frame)}
				/>
			</main>
			<Shelf
				book={book}
				labels={labels}
				selected={shown}
				onSelect={(index) => {
					setEditing(null);
					setSpreadIndex(index);
				}}
				onAdd={() => {
					setEditing(null);
					pageOp('add');
				}}
				onCommand={(command, index) => {
					const intent = shelfIntent(command, book, index);
					if (!intent) return;
					setEditing(null);
					apply(intent);
					if (command === 'duplicate' || command === 'moveRight') setSpreadIndex(index + 1);
					else if (command === 'moveLeft') setSpreadIndex(index - 1);
					else if (command === 'remove') setSpreadIndex(Math.max(0, index - 1));
				}}
				onMove={(from, to) => {
					const moved = book.spreads[from];
					if (!moved) return;
					setEditing(null);
					apply({ type: 'MoveSpread', spread: moved.id, to });
					// Nothing goes in front of a cover, and the selection follows the spread.
					setSpreadIndex(Math.max(book.spreads[0]?.kind === 'cover' ? 1 : 0, to));
				}}
			/>
		</>
	);
}
