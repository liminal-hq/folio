// The open book as the main window shows it: the stage and the inspector over the page shelf
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RibbonCommand } from '@liminal-hq/folio-ribbon/types';
import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react';
import { usePaintPresets } from './usePaintPresets';
import { useFontChoices } from './useFontChoices';
import { useFontsInUse } from './useFontsInUse';
import { shelfLabels } from '../books/shelfLabels';
import type { OpenBookCopy } from '../books/booksStore';
import { useBooks } from '../books/useBooks';
import { backgroundAssets } from '../canvas/backgrounds';
import { frameIntents } from '../canvas/frameMenu';
import { boxAt, newFrame, type NewFrameKind } from '../canvas/newFrames';
import { spreadGeometry } from '../canvas/geometry';
import { masterFurniture, pageAt } from '../canvas/masterItems';
import { masterOfPage, masterSpread, surfaceOf } from '../canvas/masterView';
import { defaultTextStyle, styleChoices } from '../inspector/styleChoices';
import { Shelf } from '../canvas/Shelf';
import { shelfIntent } from '../canvas/shelfMenu';
import { guideIntents } from '../canvas/guides';
import { coloursOf, marksOf, marksPatch, type MarkKey, type Marks } from '../canvas/marksModel';
import { keepWords } from '../canvas/editCheckpoint';
import { CheckBookDialog } from './CheckBookDialog';
import { ExportDialog } from './ExportDialog';
import { ReaderView } from './ReaderView';
import { PageBackgroundDialog } from '../canvas/PageBackgroundDialog';
import { Stage, type ZoomApi } from '../canvas/Stage';
import type { Speller } from '../canvas/TextEditor';
import { useSelection } from '../canvas/useSelection';
import { useAssetUrls } from '../canvas/useAssetUrls';
import { useSpreadLayout } from '../canvas/useSpreadLayout';
import type { CommandId } from '../commands/appMenuModel';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { DictionaryStatus } from '@liminal-hq/folio-protocol/generated/DictionaryStatus';
import { Inspector } from '../inspector/Inspector';
import { languageChoices } from '../inspector/languageChoices';
import { PagesPanel } from '../sidebar/PagesPanel';
import { FontsPanel } from '../sidebar/FontsPanel';
import { ProofingPanel } from '../sidebar/ProofingPanel';
import type { BookSpelling } from '@liminal-hq/folio-protocol/generated/BookSpelling';
import { Sidebar, type SidebarGroup } from '../sidebar/Sidebar';
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
	/** The last cell edited in the selected table: where the ribbon puts a row or column. */
	const [editingCell, setEditingCell] = useState<{ row: number; col: number } | null>(null);
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
		settings.spelling.enabled ? settings.spelling.personal : null,
	);
	// What the stage shows: the pages as printed, or as a Kindle or a Kobo shows them.
	const [preview, setPreview] = useState<'print' | 'kindle' | 'kobo'>('print');
	const [selection, select] = useSelection(
		spread?.id ?? null,
		spread?.frames.map((f) => f.id) ?? [],
	);
	const images = useAssetUrls(client, id, [
		...[...(spread?.frames ?? []), ...(spread ? masterFurniture(book, spread) : [])].flatMap((f) =>
			f.type === 'image' && f.image.asset ? [f.image.asset] : [],
		),
		// The shelf draws every spread's background, so every picture one uses is read.
		...backgroundAssets(book),
	]);
	const choices = useMemo(() => styleChoices(copy.styles), [copy.styles]);
	const fonts = useFontChoices(client, id, (book.fonts ?? []).map((f) => f.path).join('|'));
	const selected = useMemo(
		() => spread?.frames.filter((f) => selection.includes(f.id)) ?? [],
		[spread, selection],
	);
	// A cell belongs to the table it was edited in, so choosing something else forgets it.
	useEffect(() => setEditingCell(null), [selection.join('|')]);
	// Spelling is checked in Rust, in the book's own language and against its ignore list.
	const spelling = settings.spelling.enabled;
	const personal = settings.spelling.personal;
	const speller = useMemo<Speller | undefined>(
		() =>
			spelling
				? {
						check: (text) =>
							client.spellCheck(id, text, personal).then((found) => found.misspellings),
						suggest: (word) => client.spellSuggest(id, word),
						ignore: (word) => void store.apply({ type: 'IgnoreWord', word }),
						addToDictionary: (word) =>
							void update({
								spelling: { personal: [...new Set([...personal, word])].sort() },
							} as SettingsPatch),
					}
				: undefined,
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[spelling, client, id, personal],
	);
	const input: EditorInput | null = spread
		? {
				spread,
				spreadIndex: shown,
				spreadCount: book.spreads.length,
				selected,
				layout,
				choices,
				...(fonts ? { fonts } : {}),
				marks,
				snap,
				rulers: rulers.on,
				preview,
				master: master !== null,
				cell: editingCell,
			}
		: null;
	const editor = useMemo(
		() => (input ? editorState(input) : undefined),
		[
			spread,
			shown,
			book.spreads.length,
			selected,
			layout,
			choices,
			marks,
			snap,
			rulers.on,
			editingCell,
		],
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

	// Anything else that changes the book while words are being typed lands after those words: the
	// editor keeps what it has first, so undo takes the changes back in the order they happened.
	const apply = (intent: Intent) => {
		keepWords();
		void store.apply(intent);
	};
	/** Words kept from an editor are a step of their own, never joined to the typing before them. */
	const keepAsStep = (intent: Intent) => {
		void store.apply({ type: 'EndUndoStep' });
		void store.apply(intent);
	};

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

	/** Asks for a picture and brings it into the book; its path, or `null` if none was chosen. */
	const pickBackgroundPicture = async (): Promise<string | null> => {
		const picked = await dialogs.pickImage();
		if (picked === null) return null;
		try {
			const asset = await client.importImage(id, picked);
			return store.state.getState().activeId === id ? asset : null;
		} catch {
			return null;
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

	// The Check book dialog, and the frame to select once "Show me" has gone to its spread.
	const [checking, setChecking] = useState(false);
	// The Export dialog; `quick` makes the Sharing PDF the moment it opens.
	const [exporting, setExporting] = useState<{ quick: boolean } | null>(null);
	const [reveal, setReveal] = useState<{ spread: string; frame: string } | null>(null);
	useEffect(() => {
		if (reveal && spread?.id === reveal.spread) {
			select([reveal.frame]);
			setReveal(null);
		}
	});
	// The page background dialog: which page it was asked about, and the palette it offers.
	const [background, setBackground] = useState<{ page: number | null; scope?: string } | null>(
		null,
	);

	// The page background dialog applies each try at once. Cancelling rewinds the book to how many steps
	// it could undo when the dialog opened.
	const backgroundOpen = background !== null;
	const backgroundMark = useRef<number | null>(null);
	useEffect(() => {
		if (!backgroundOpen) return;
		backgroundMark.current = null;
		let live = true;
		void store.mark().then((mark) => {
			if (live) backgroundMark.current = mark;
		});
		return () => {
			live = false;
		};
	}, [backgroundOpen, store]);
	const cancelBackground = () => {
		const mark = backgroundMark.current;
		setBackground(null);
		if (mark !== null) void store.rewind(mark);
	};
	// The sidebar's open group. Selecting something on the page brings the Properties forward.
	const [group, setGroup] = useState<SidebarGroup>('properties');
	const fontUses = useFontsInUse(client, id, copy.revision, group === 'fonts');
	useEffect(() => {
		// The Proofing list is a task: showing a word selects it, and the list stays.
		if (selection.length > 0) setGroup((open) => (open === 'proofing' ? open : 'properties'));
	}, [selection]);
	// What spelling finds in the whole book, read when the Proofing group is open and read again a
	// moment after the book, the dictionary or the words added for every book change.
	const [proofing, setProofing] = useState<BookSpelling | null>(null);
	const [dictionaryTick, setDictionaryTick] = useState(0);
	const personalKey = settings.spelling.personal.join('\u0000');
	// The dictionaries there are, for the language list: read once and again after a download.
	const [dictionaries, setDictionaries] = useState<DictionaryStatus[]>([]);
	useEffect(() => {
		let live = true;
		client
			.dictionaries()
			.then((found) => live && setDictionaries(found))
			.catch(() => undefined);
		return () => {
			live = false;
		};
	}, [client, dictionaryTick]);
	const languages = useMemo(
		() => languageChoices(dictionaries, book.language),
		[dictionaries, book.language],
	);
	useEffect(() => {
		if (group !== 'proofing') return;
		let live = true;
		const wait = setTimeout(() => {
			client
				.spellCheckBook(id, settings.spelling.personal)
				.then((found) => live && setProofing(found))
				.catch(
					() =>
						live &&
						setProofing({
							supported: false,
							dictionary: null,
							offer: null,
							words: [],
							truncated: false,
						}),
				);
		}, 300);
		return () => {
			live = false;
			clearTimeout(wait);
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [group, client, id, copy.revision, personalKey, dictionaryTick]);
	// The colours the pickers offer, read the first time the dialog opens or a shape is selected.
	const paints = usePaintPresets(
		client,
		background !== null || (group === 'properties' && selected.some((f) => f.type === 'shape')),
	);

	const perform = (action: EditorAction) => {
		switch (action.kind) {
			case 'intents':
				action.intents.forEach(apply);
				break;
			case 'font':
				action.intents.forEach(apply);
				// An installed font is kept inside the book, so it opens the same on another computer.
				if (action.keep) void client.keepFonts(id, [action.keep]).catch(() => undefined);
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
			case 'spelling':
				void update({ spelling: { enabled: !settings.spelling.enabled } } as SettingsPatch);
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
			case 'pageBackground':
				setBackground({ page: null });
				break;
			case 'preview':
				setPreview(action.mode);
				break;
			case 'checkBook':
				setChecking(true);
				break;
			case 'exportBook':
				setExporting({ quick: action.quick });
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
					{preview === 'print' ? (
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
							onTurn={(frame, degrees) =>
								store.apply({ type: 'RotateFrame', on: surface, frame, degrees })
							}
							onEditText={(frame, text) => keepAsStep({ type: 'SetText', frame, text })}
							onEditTable={(frame, edit) => keepAsStep({ type: 'EditTable', frame, edit })}
							speller={speller}
							onEditingCell={(cell) => cell && setEditingCell(cell)}
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
								else if (command === 'pageBackground')
									setBackground({ page: pageAt(book, spread, point) });
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
					) : (
						<ReaderView
							client={client}
							id={id}
							book={book}
							revision={copy.revision}
							device={preview}
							spread={spread}
							layout={layout}
							images={images}
						/>
					)}
				</div>
				<Sidebar
					group={group}
					onGroup={setGroup}
					panels={{
						properties: (
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
								paints={paints}
								images={images}
								choosePicture={(frame) => void choosePicture(frame)}
								languages={languages}
							/>
						),
						proofing: (
							<ProofingPanel
								found={proofing}
								spreads={book.spreads.map((s, i) => ({ id: s.id, label: labels[i] ?? '' }))}
								onShow={(word) => {
									const index = book.spreads.findIndex((s) => s.id === word.spread);
									if (index < 0) return;
									setEditing(null);
									setSpreadIndex(index);
									select([word.frame]);
								}}
								languages={languages}
								onLanguage={(language) => void store.apply({ type: 'SetLanguage', language })}
								ignored={book.proofing?.ignore ?? []}
								onRestore={(word) => void store.apply({ type: 'UnignoreWord', word })}
								onIgnore={(word) => void store.apply({ type: 'IgnoreWord', word: word.word })}
								onAdd={(word) =>
									void update({
										spelling: { personal: [...new Set([...personal, word.word])].sort() },
									} as SettingsPatch)
								}
								onSuggest={(word) => client.spellSuggest(id, word).catch(() => [])}
								onChange={(word, to) => {
									const frame = book.spreads
										.flatMap((s) => s.frames)
										.find((f) => f.id === word.frame);
									if (!frame) return;
									const words = (content?: { inline: { text: string }[] } | null) =>
										content?.inline.map((r) => r.text).join('') ?? '';
									const swap = (text: string) =>
										text.slice(0, word.start) + to + text.slice(word.end);
									if (frame.type === 'table' && word.row !== null && word.col !== null) {
										const cell = frame.rows[word.row]?.[word.col];
										if (!cell) return;
										void store.apply({
											type: 'EditTable',
											frame: frame.id,
											edit: {
												type: 'setCell',
												row: word.row,
												col: word.col,
												text: swap(words(cell.content)),
											},
										});
									} else if (frame.type === 'text' || frame.type === 'shape') {
										void store.apply({
											type: 'SetText',
											frame: frame.id,
											text: swap(words(frame.content)),
										});
									}
								}}
								onDownload={(dictionary) =>
									void client
										.downloadDictionary(dictionary)
										.then(() => setDictionaryTick((n) => n + 1))
								}
							/>
						),
						fonts: (
							<FontsPanel
								uses={fontUses}
								onKeep={(family) => void client.keepFonts(id, [family]).catch(() => undefined)}
								onLetGo={(family) => void client.unkeepFonts(id, [family]).catch(() => undefined)}
							/>
						),
						pages: (
							<PagesPanel
								book={book}
								labels={labels}
								shown={shown}
								images={images}
								onShow={(index) => {
									setEditing(null);
									setSpreadIndex(index);
								}}
								onChange={(scope) => setBackground({ page: null, scope })}
								onApply={apply}
							/>
						),
					}}
				/>
			</main>
			<Shelf
				book={book}
				images={images}
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
			<ExportDialog
				open={exporting !== null}
				onClose={() => setExporting(null)}
				client={client}
				id={id}
				autoStart={exporting?.quick ? ['sharingPdf'] : null}
				onCheck={() => setChecking(true)}
			/>
			<CheckBookDialog
				open={checking}
				onClose={() => setChecking(false)}
				client={client}
				id={id}
				book={book}
				labels={labels}
				onShow={(at, frame) => {
					if (master !== null) setEditing(null);
					setSpreadIndex(at);
					const target = book.spreads[at];
					if (frame && target) setReveal({ spread: target.id, frame });
				}}
			/>
			<PageBackgroundDialog
				open={background !== null}
				onClose={() => setBackground(null)}
				onCancel={cancelBackground}
				book={book}
				spread={spread}
				page={background?.page ?? null}
				scope={background?.scope ?? null}
				presets={paints}
				images={images}
				onPickPicture={pickBackgroundPicture}
				onApply={apply}
			/>
		</>
	);
}
