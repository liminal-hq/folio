// The open book as the main window shows it: the stage and the inspector over the page shelf
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RibbonCommand } from '@liminal-hq/folio-ribbon/types';
import {
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
	useSyncExternalStore,
	type CSSProperties,
} from 'react';
import { usePaintPresets } from './usePaintPresets';
import { useFontChoices } from './useFontChoices';
import { useFontsInUse } from './useFontsInUse';
import { resolveSwatches, swatchDisplays } from '../books/swatches';
import { shelfLabels } from '../books/shelfLabels';
import type { OpenBookCopy } from '../books/booksStore';
import { useBooks } from '../books/useBooks';
import { backgroundAssets } from '../canvas/backgrounds';
import { pageColourOf } from '../colour/pageColour';
import { CLIPBOARD_COMMANDS, GROUP_COMMANDS, frameIntents } from '../canvas/frameMenu';
import { canGroup, canUngroup, childIds, groupIntent, ungroupIntent } from '../canvas/grouping';
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
import { dialogLabel, type DialogMessages } from '../dialogs/dialogKinds';
import { tauriDialogsClient } from '../dialogs/dialogsClient';
import { openDialog } from '../dialogs/openDialog';
import { putDescriptionIntent } from './coverSheetIntents';
import { useSpeller } from './useSpeller';
import { useWindowedDialog } from '../dialogs/useWindowedDialog';
import { ExportDialog } from './ExportDialog';
import {
	getPendingLook,
	getTextCaret,
	getTextLookState,
	getTextParagraphs,
	getTextSelection,
	setPendingLook,
	subscribeTextLook,
	subscribeTextParagraphs,
} from '../canvas/textSelection';
import { IMPORT_TEXT_EVENT } from './importRequest';
import { ImportReportDialog, type ImportOutcome } from './ImportReportDialog';
import { StoryEditorDialog } from '../story/StoryEditorDialog';
import { rangeIntents } from '../story/rangeEdit';
import { BookDetailsDialog } from './BookDetailsDialog';
import type { PastedPicture } from '../clipboard/frameClipboard';
import { useFrameClipboard } from '../clipboard/useFrameClipboard';
import { openEditor } from '../text/openEditor';
import { BarcodeDialog } from './BarcodeDialog';
import { BookSizeDialog } from './BookSizeDialog';
import { MarginsDialog } from './MarginsDialog';
import { LinkDialog } from './LinkDialog';
import { openUrl } from '@tauri-apps/plugin-opener';
import { setLinkActions } from '../text/linkActions';
import { isExternalHref, spreadOfHref } from '../text/links';
import {
	linkIntent,
	linkTarget,
	textBetween,
	type LinkTarget,
	type LookScope,
} from '../ribbon/selectionLook';
import { SpreadTemplatesDialog } from './SpreadTemplatesDialog';
import { CoverSheetDialog } from './CoverSheetDialog';
import { JAX_TOGGLE_EVENT } from '../jax/jaxToggle';
import { JaxPanel } from '../jax/JaxPanel';
import rowStyles from '../jax/shelfSlot.module.css';
import { SoftProofDialog } from './SoftProofDialog';
import { EXPORT_EVENT } from './exportRequest';
import { FindBar } from '../find/FindBar';
import { styleForKey } from './useBookKeys';
import { FIND_EVENT } from '../find/findRequest';
import { onShown } from '../find/matches';
import { useFind } from '../find/useFind';
import { ReaderView } from './ReaderView';
import { PageBackgroundDialog } from '../canvas/PageBackgroundDialog';
import { Stage, type ZoomApi } from '../canvas/Stage';
import { useSelection } from '../canvas/useSelection';
import { useAssetUrls } from '../canvas/useAssetUrls';
import { useCoverSheet } from '../canvas/useCoverSheet';
import { useSpreadLayout } from '../canvas/useSpreadLayout';
import { useTextEdits } from '../text/useTextEdits';
import type { CommandId } from '../commands/appMenuModel';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { DictionaryStatus } from '@liminal-hq/folio-protocol/generated/DictionaryStatus';
import { Inspector } from '../inspector/Inspector';
import { languageChoices } from '../inspector/languageChoices';
import { PagesPanel } from '../sidebar/PagesPanel';
import { FontsPanel } from '../sidebar/FontsPanel';
import { useStoryChapters } from './useStoryChapters';
import { ProofingPanel } from '../sidebar/ProofingPanel';
import type { BookSpelling } from '@liminal-hq/folio-protocol/generated/BookSpelling';
import { EXPERT_GROUPS, Sidebar, type SidebarGroup } from '../sidebar/Sidebar';
import { SwatchesPanel } from '../sidebar/SwatchesPanel';
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
	// The cover is the printed sheet: what Folio draws on it (the barcode, the spine) goes over the
	// frames the book holds, and the folds and safe areas are guides after the person's own.
	const sheet = useCoverSheet(
		client,
		id,
		copy.revision,
		master === null && spread?.kind === 'cover',
		useMemo(() => new Set(spread?.frames.map((f) => f.id)), [spread]),
	);
	const onSheet = useMemo(
		() =>
			sheet && spread
				? {
						...spread,
						frames: [...spread.frames, ...sheet.frames],
						guides: [...spread.guides, ...sheet.guides],
					}
				: spread,
		[spread, sheet],
	);
	// A barcode box with no barcode in it says it can be made, in grey, and never prints.
	const barcodeHint = useMemo(() => {
		const box = onSheet?.frames.find((f) => f.id === 'cover_barcode')?.box;
		const drawn = onSheet?.frames.some((f) => f.id.startsWith('cover_bar_'));
		return box && !drawn ? { box, text: t('cover.barcodeHint') } : undefined;
	}, [onSheet]);
	// What is drawn has each reference to a swatch turned into the colour the swatch shows as.
	const displays = useMemo(() => swatchDisplays(book), [book.styles]);
	const drawn = useMemo(() => resolveSwatches(book, displays), [book, displays]);
	const drawnSpread = useMemo(
		() => (onSheet ? resolveSwatches(onSheet, displays) : onSheet),
		[onSheet, displays],
	);
	const textEdits = useTextEdits(
		store,
		spread?.id ?? null,
		settings.spelling.enabled ? settings.spelling.personal : null,
		settings.editing.smartPunctuation,
	);
	const spreadLayout = useSpreadLayout(
		client,
		id,
		master ? editing : (spread?.id ?? null),
		copy.revision,
		master !== null,
		settings.spelling.enabled ? settings.spelling.personal : null,
		textEdits.seed,
	);
	const layout = sheet?.layout ?? spreadLayout;
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
	// The sidebar's open group. Selecting something on the page brings the Properties forward.
	const [group, setGroup] = useState<SidebarGroup>('properties');
	// The Font list wants the families in use whichever group is open. It asks again when the styles or
	// the kept fonts change, not on every edit; the Fonts group asks on every one, as it shows them.
	const stylesKey = useMemo(() => JSON.stringify(copy.styles ?? null), [copy.styles]);
	const fontUses = useFontsInUse(
		client,
		id,
		group === 'fonts' ? copy.revision : `${stylesKey}|${(book.fonts ?? []).length}`,
		true,
	);
	const selected = useMemo(
		() => spread?.frames.filter((f) => selection.includes(f.id)) ?? [],
		[spread, selection],
	);
	// A cell belongs to the table it was edited in, so choosing something else forgets it.
	useEffect(() => setEditingCell(null), [selection.join('|')]);
	// Spelling is checked in Rust, in the book's own language and against its ignore list.
	const personal = settings.spelling.personal;
	const speller = useSpeller(client, id, store.apply);
	const [dictionaryTick, setDictionaryTick] = useState(0);
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
	// The paragraphs the open editor's caret is in, which the Paragraph controls act on.
	const textParagraphs = useSyncExternalStore(subscribeTextParagraphs, getTextParagraphs);
	// What is selected or under the caret, so the controls show the look of those words (Expert only).
	const textLook = useSyncExternalStore(subscribeTextLook, getTextLookState);
	const expert = settings.mode === 'expert';
	const input: EditorInput | null = spread
		? {
				spread,
				spreadIndex: shown,
				spreadCount: book.spreads.length,
				selected,
				layout,
				choices,
				...(fonts ? { fonts } : {}),
				...(fontUses ? { fontsInUse: fontUses.map((u) => u.family) } : {}),
				languages: languages.choices,
				marks,
				snap,
				rulers: rulers.on,
				preview,
				hasCover: book.spreads[0]?.kind === 'cover',
				pages: spreadGeometry(book, spread).pages.map((p) => p.trim),
				jax: settings.jax !== 'off',
				master: master !== null,
				cell: editingCell,
				swatches: book.styles?.swatches ?? [],
				pageColour: pageColourOf(spread, book),
				paragraphs: textParagraphs,
				// Copy and Cut take the words selected in the open editor, not the frame.
				words:
					textLook.caret || textLook.selection ? { selected: textLook.selection !== null } : null,
				flows: book.flows ?? [],
				selection: expert ? textLook.selection : null,
				caret: expert ? textLook.caret : null,
				pending: expert && textLook.caret ? getPendingLook(textLook.caret.frame) : null,
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
			fonts,
			fontUses,
			marks,
			snap,
			rulers.on,
			settings.jax,
			book.spreads[0]?.kind,
			book.trim,
			book.margins,
			editingCell,
			book.styles,
			book.flows,
			textParagraphs,
			textLook,
			expert,
			languages,
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

	// Ctrl+B, Ctrl+I and Ctrl+U in the on-page editor style the selected words (or the next ones typed
	// at a caret), as the ribbon's buttons do. Only the editor's own input is listened to.
	const styleKey = useRef<(id: string) => void>(() => undefined);
	styleKey.current = (name) => {
		if (settings.mode === 'expert') onEditorCommand({ id: name });
	};
	useEffect(() => {
		const onKey = (event: KeyboardEvent) => {
			const name = styleForKey(event);
			if (!name) return;
			if (!(event.target instanceof HTMLElement) || !event.target.closest('[data-text-sink]'))
				return;
			event.preventDefault();
			styleKey.current(name);
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	}, []);

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

	/** The frames just grouped, until the group they went into appears and can be selected. */
	const [grouping, setGrouping] = useState<string[] | null>(null);
	const makeGroup = (frames: Frame[]) => {
		if (!canGroup(frames, master !== null)) return;
		apply(groupIntent(frames, surface));
		setGrouping(frames.map((f) => f.id));
	};
	const breakGroup = (group: Frame) => {
		if (!canUngroup([group])) return;
		apply(ungroupIntent(group, surface));
		// Its frames stay where they are, and are chosen once they are back among the page's own.
		select(childIds(group));
	};
	useEffect(() => {
		if (!grouping || !spread) return;
		const made = spread.frames.find(
			(f) => f.type === 'group' && grouping.every((id) => f.children.some((c) => c.id === id)),
		);
		if (made) {
			select([made.id]);
			setGrouping(null);
		}
	});
	const clipboard = useFrameClipboard({
		active: preview === 'print',
		selected,
		surface,
		apply,
		select,
		pastePicture: (picture, into) => void pastePicture(picture, into),
		pasteText: (text, into) => pasteText(text, into),
		say: (message) => setNotice(message),
		nothingMessage: t('clipboard.nothing'),
	});

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
		return frame;
	};

	/** Brings a picture the clipboard held into the book and puts it on the page as an Add → Picture frame. */
	const pastePicture = async ({ file, extension }: PastedPicture, into?: string) => {
		try {
			const asset = await client.importImageBytes(
				id,
				new Uint8Array(await file.arrayBuffer()),
				extension,
			);
			// The book on show may have changed while the picture was read.
			if (store.state.getState().activeId !== id) return;
			const frame = into ? { id: into } : addFrame('picture');
			await store.apply({ type: 'SetImage', frame: frame.id, asset, fit: null });
		} catch {
			setNotice(t('clipboard.pictureRefused'));
		}
	};

	/** Puts words on the page in a new text box, or as the words of the frame `into`. */
	const pasteText = (text: string, into?: string) => {
		if (into) {
			apply({ type: 'SetText', frame: into, text });
			return;
		}
		const pages = spreadGeometry(book, spread).pages;
		const first = pages[0]!;
		const frame = newFrame(
			'text',
			boxAt(
				'text',
				{ x: first.trim.x + first.trim.w / 2, y: first.trim.y + first.trim.h / 2 },
				first.trim,
			),
			defaultTextStyle(choices),
		);
		apply({
			type: 'AddFrame',
			on: surface,
			frame: { ...frame, content: { inline: [{ text }] } } as unknown as Frame,
		});
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
	const [proofOpen, setProofOpen] = useState(false);
	const [sheetOpen, setSheetOpen] = useState(false);
	/** Jax's panel, open from the start when the person asked to be walked through. */
	const [jaxOpen, setJaxOpen] = useState(settings.jax === 'guide');
	useEffect(() => {
		const toggle = () => setJaxOpen((open) => !open);
		window.addEventListener(JAX_TOGGLE_EVENT, toggle);
		return () => window.removeEventListener(JAX_TOGGLE_EVENT, toggle);
	}, []);
	const [barcodeOpen, setBarcodeOpen] = useState(false);
	const [bookSizeOpen, setBookSizeOpen] = useState(false);
	const [marginsOpen, setMarginsOpen] = useState(false);
	/** The text frame whose words are to continue in the next frame clicked. */
	const [linking, setLinking] = useState<string | null>(null);
	/** What the last text import did, or why it could not. */
	const [imported, setImported] = useState<ImportOutcome | null>(null);
	/** The file last brought in, so it can be brought in again with other choices of style. */
	const importedPath = useRef<string | null>(null);
	const smartPunctuation = settings.editing.smartPunctuation;
	const bringIn = useCallback(
		async (path: string, mapping?: Record<string, string>) => {
			try {
				setImported({
					report: await client.importText(id, path, mapping, smartPunctuation),
				});
			} catch (reason) {
				setImported({
					problem:
						typeof reason === 'object' && reason !== null && 'message' in reason
							? String((reason as { message: unknown }).message)
							: String(reason),
				});
			}
		},
		[client, id, smartPunctuation],
	);
	useEffect(() => {
		const run = () => {
			void (async () => {
				const path = await dialogs.pickText();
				if (path === null) return;
				importedPath.current = path;
				await bringIn(path);
			})();
		};
		window.addEventListener(IMPORT_TEXT_EVENT, run);
		return () => window.removeEventListener(IMPORT_TEXT_EVENT, run);
	}, [bringIn, dialogs]);
	/** The flow whose story is open in the story editor. */
	const [storyFlow, setStoryFlow] = useState<string | null>(null);
	/** The paragraph the story editor opens at, when it was opened from a frame of the story. */
	const [storyPara, setStoryPara] = useState<number | null>(null);
	// Escape gives up choosing the frame the words continue in.
	useEffect(() => {
		if (linking === null) return;
		const onKey = (event: KeyboardEvent) => {
			if (event.key === 'Escape') setLinking(null);
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	}, [linking]);
	const [templatesOpen, setTemplatesOpen] = useState(false);
	/** What the last change from a dialog said, shown over the page for a few seconds. */
	const [notice, setNotice] = useState<string | null>(null);
	useEffect(() => {
		if (notice === null) return;
		const timer = setTimeout(() => setNotice(null), 6000);
		return () => clearTimeout(timer);
	}, [notice]);
	const [detailsOpen, setDetailsOpen] = useState(false);
	const [background, setBackground] = useState<{ page: number | null; scope?: string } | null>(
		null,
	);
	// The Export dialog; `quick` makes the Sharing PDF the moment it opens.
	const [exporting, setExporting] = useState<{ quick: boolean } | null>(null);
	useEffect(() => {
		const open = () => setExporting({ quick: false });
		window.addEventListener(EXPORT_EVENT, open);
		return () => window.removeEventListener(EXPORT_EVENT, open);
	}, []);
	// Find and replace: the bar is opened by Ctrl+F and Ctrl+H (and the ribbon's Find & replace), and
	// the page goes to the match it is on.
	const find = useFind(client, id, copy.revision, editing);
	useEffect(() => {
		const open = (event: Event) =>
			find.show((event as CustomEvent<{ replace?: boolean }>).detail?.replace === true);
		window.addEventListener(FIND_EVENT, open);
		return () => window.removeEventListener(FIND_EVENT, open);
	}, [find.show]);
	const foundAt = find.focus?.match;
	useEffect(() => {
		if (!foundAt) return;
		if (foundAt.master) {
			if (masterSpread(book, foundAt.master)) setEditing(foundAt.master);
		} else {
			const at = book.spreads.findIndex((s) => s.id === foundAt.spread);
			if (at >= 0) {
				setEditing(null);
				setSpreadIndex(at);
			}
		}
		// Only a new request moves the page; the book changing under it does not.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [find.focus?.nonce]);
	const closeFind = () => {
		find.close();
		// The words being edited, or the page, get the keyboard back.
		queueMicrotask(() =>
			(
				document.querySelector<HTMLElement>('[data-text-sink]') ??
				document.querySelector<HTMLElement>('svg[role="group"]')
			)?.focus(),
		);
	};
	const marked =
		find.open && spread ? find.matches.filter((m) => onShown(m, spread.id, editing)) : [];
	// With dialogs in their own windows the Export dialog is one, and it hands back "check the book
	// first" as its only answer.
	const dialogWindows = settings.dialogs.ownWindows;
	useEffect(() => {
		if (!dialogWindows || exporting === null) return;
		const asked = exporting;
		setExporting(null);
		openDialog('Export', { bookId: id, autoStart: asked.quick ? ['sharingPdf'] : null })
			.then((answer) => {
				if (answer?.next === 'check') setChecking(true);
			})
			.catch(() => {
				// The window could not be made.
			});
	}, [dialogWindows, exporting, id]);
	const bookDialog = { bookId: id };
	const showDetails = useWindowedDialog('BookDetails', detailsOpen, dialogWindows, bookDialog, () =>
		setDetailsOpen(false),
	);
	const showBarcode = useWindowedDialog('Barcode', barcodeOpen, dialogWindows, bookDialog, () =>
		setBarcodeOpen(false),
	);
	const showBookSize = useWindowedDialog('BookSize', bookSizeOpen, dialogWindows, bookDialog, () =>
		setBookSizeOpen(false),
	);
	const showMargins = useWindowedDialog('Margins', marginsOpen, dialogWindows, bookDialog, () =>
		setMarginsOpen(false),
	);
	const showProof = useWindowedDialog(
		'SoftProof',
		proofOpen,
		dialogWindows,
		{ bookId: id, spread: spread?.id ?? '' },
		() => setProofOpen(false),
	);
	// Check book is a window beside the book that stays open; "Show me" in it posts here.
	const showCheck = useWindowedDialog('CheckBook', checking, dialogWindows, bookDialog, () =>
		setChecking(false),
	);
	const showIssueRef = useRef<(at: number, frame: string | null) => void>(() => {});
	useEffect(() => {
		if (!dialogWindows) return;
		let live = true;
		let stop: (() => void) | undefined;
		tauriDialogsClient
			.onMessage((posted) => {
				if (posted.label !== dialogLabel('CheckBook')) return;
				const message = posted.message as DialogMessages['CheckBook'];
				if (message.type === 'show') showIssueRef.current(message.spread, message.frame);
			})
			.then((off) => (live ? (stop = off) : off()))
			.catch(() => {});
		return () => {
			live = false;
			stop?.();
		};
	}, [dialogWindows]);
	const showBackground = useWindowedDialog(
		'PageBackground',
		background !== null,
		dialogWindows,
		{
			bookId: id,
			spread: spread?.id ?? '',
			page: background?.page ?? null,
			scope: background?.scope ?? null,
		},
		() => setBackground(null),
	);
	const showStory = useWindowedDialog(
		'StoryEditor',
		storyFlow !== null,
		dialogWindows,
		{ bookId: id, flow: storyFlow ?? '', paragraph: storyPara },
		() => setStoryFlow(null),
	);
	const showSheet = useWindowedDialog('CoverSheet', sheetOpen, dialogWindows, bookDialog, () =>
		setSheetOpen(false),
	);
	// The spread on show decides where a template goes, so the window hands the choice back.
	const addTemplateRef = useRef<(template: string, name: string) => Promise<void>>(async () => {});
	const showTemplates = useWindowedDialog(
		'SpreadTemplates',
		templatesOpen,
		dialogWindows,
		bookDialog,
		(chosen) => {
			setTemplatesOpen(false);
			if (chosen) {
				addTemplateRef.current(chosen.template, chosen.name).catch(() => {
					setNotice(t('templates.refused'));
				});
			}
		},
	);
	useWindowedDialog(
		'ImportReport',
		imported !== null,
		dialogWindows,
		{ outcome: imported as ImportOutcome },
		(answer) => {
			setImported(null);
			if (answer && importedPath.current !== null)
				void bringIn(importedPath.current, answer.mapping);
		},
	);
	/** Goes to a spread and, when the issue is about a frame, selects it. */
	const showIssue = (at: number, frame: string | null) => {
		if (master !== null) setEditing(null);
		setSpreadIndex(at);
		const target = book.spreads[at];
		if (frame && target) setReveal({ spread: target.id, frame });
	};
	showIssueRef.current = showIssue;
	/** Adds a template after the spread on show, or after the first page when the cover is. */
	const addTemplate = async (template: string, name: string) => {
		const index = spread?.kind === 'cover' ? 1 : shown;
		const after = book.spreads[index];
		if (!after) throw new Error(t('templates.refused'));
		await client.applyTemplate(id, template, after.id);
		setSpreadIndex(index + 1);
		select([]);
		setNotice(t('templates.added', { name }));
	};
	addTemplateRef.current = addTemplate;
	const [reveal, setReveal] = useState<{ spread: string; frame: string } | null>(null);
	useEffect(() => {
		if (reveal && spread?.id === reveal.spread) {
			select([reveal.frame]);
			setReveal(null);
		}
	});
	// The page background dialog: which page it was asked about, and the palette it offers.

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
	const chapters = useStoryChapters(client, id, copy.revision, group === 'pages');
	// `useSelection` hands back a new array on every render, so the effect keys on the ids themselves:
	// opening Fonts or Swatches with a frame selected must not bring Properties back.
	const selectionKey = selection.join('|');
	useEffect(() => {
		// The Proofing list is a task: showing a word selects it, and the list stays.
		if (selectionKey !== '') setGroup((open) => (open === 'proofing' ? open : 'properties'));
	}, [selectionKey]);
	// What spelling finds in the whole book, read when the Proofing group is open and read again a
	// moment after the book, the dictionary or the words added for every book change.
	const [proofing, setProofing] = useState<BookSpelling | null>(null);
	const personalKey = settings.spelling.personal.join('\u0000');
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

	/** The words the link dialog is for: where they are, the link they have and what they say. */
	const [linkDialog, setLinkDialog] = useState<{
		scope: LookScope;
		target: LinkTarget;
		words: string;
	} | null>(null);
	const openLinkFor = (scope: LookScope) => {
		const target = linkTarget(scope);
		setLinkDialog({ scope, target, words: textBetween(scope, target.from, target.to) });
	};
	/** Opens the link dialog for what the on-page editor has selected (Ctrl+K, in Easy as well as Expert). */
	const openLink = () => {
		const caret = getTextCaret();
		const action = editorAction('link', undefined, {
			...input!,
			selection: getTextSelection(),
			caret,
			pending: null,
		});
		if (action?.kind === 'link') openLinkFor(action.scope);
	};
	/** Follows a link: an address opens in the system's program, a spread of the book is shown. */
	const followLink = (href: string) => {
		const spreadNumber = spreadOfHref(href);
		if (spreadNumber !== null) {
			if (editing !== null) setEditing(null);
			setSpreadIndex(Math.min(spreadNumber, book.spreads.length) - 1);
			select([]);
		} else if (isExternalHref(href)) {
			openUrl(href).catch(() => setNotice(t('link.openFailed')));
		}
	};
	const linkNow = useRef<{
		followLink(href: string): void;
		openLink(): void;
		removeLink(): void;
	}>(null!);
	linkNow.current = {
		followLink,
		openLink,
		removeLink: () => {
			const action = editorAction('link', undefined, {
				...input!,
				selection: getTextSelection(),
				caret: getTextCaret(),
				pending: null,
			});
			if (action?.kind !== 'link') return;
			apply(linkIntent(action.scope, linkTarget(action.scope), ''));
		},
	};
	useEffect(
		() =>
			setLinkActions({
				follow: (href) => linkNow.current.followLink(href),
				edit: () => linkNow.current.openLink(),
				remove: () => linkNow.current.removeLink(),
			}),
		[],
	);
	// Ctrl+K in the on-page editor sets, changes or takes away the link on the selected words.
	useEffect(() => {
		const onKey = (event: KeyboardEvent) => {
			if (!(event.ctrlKey || event.metaKey) || event.altKey || event.shiftKey) return;
			if (event.key.toLowerCase() !== 'k') return;
			if (!(event.target instanceof HTMLElement) || !event.target.closest('[data-text-sink]'))
				return;
			event.preventDefault();
			linkNow.current.openLink();
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	}, []);

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
			case 'pending':
				setPendingLook(action.frame, action.change);
				// An installed font is kept inside the book, so it opens the same on another computer.
				if (action.keep) void client.keepFonts(id, [action.keep]).catch(() => undefined);
				break;
			case 'find':
				find.show(false);
				break;
			case 'link':
				openLinkFor(action.scope);
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
			case 'templates':
				setTemplatesOpen(true);
				break;
			case 'softProof':
				setProofOpen(true);
				break;
			case 'coverSheet':
				setSheetOpen(true);
				break;
			case 'barcode':
				setBarcodeOpen(true);
				break;
			case 'bookDetails':
				setDetailsOpen(true);
				break;
			case 'exportBook':
				setExporting({ quick: action.quick });
				break;
			case 'rulerUnit':
				void update({ rulers: { units: action.unit } } as SettingsPatch);
				break;
			case 'toolbar':
				void update({ interface: { toolbar: action.toolbar } } as SettingsPatch);
				break;
			case 'bookSize':
				setBookSizeOpen(true);
				break;
			case 'pageMargins':
				setMarginsOpen(true);
				break;
			case 'cover':
				// The cover is the first spread; a master being edited gives way to it.
				setEditing(null);
				setSpreadIndex(0);
				select([]);
				break;
			case 'clipboard': {
				// While the on-page editor is open, the clipboard is for its words, as Ctrl+C and Ctrl+V are.
				const words = openEditor();
				if (words && action.op !== 'pasteInPlace') {
					if (action.op === 'copy') void words.copy();
					else if (action.op === 'cut') void words.cut();
					else
						void words.paste(action.op === 'pastePlain').then((pasted) => {
							if (!pasted) setNotice(t('clipboard.nothing'));
						});
					break;
				}
				if (action.op === 'copy') void clipboard.copy();
				else if (action.op === 'cut') void clipboard.cut();
				else if (action.op === 'paste') void clipboard.pasteWith('paste');
				else if (action.op === 'pasteInPlace') void clipboard.pasteWith('inPlace');
				else if (action.op === 'pastePlain') void clipboard.pasteWith('plain');
				else void clipboard.pasteWith('into');
				break;
			}
			case 'group':
				makeGroup(action.frames);
				break;
			case 'ungroup':
				breakGroup(action.frame);
				break;
			case 'frameLink':
				// The Properties group says to choose the box the words continue in.
				setGroup('properties');
				setLinking(action.frame);
				break;
			case 'jax':
				setJaxOpen((open) => !open);
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
		// What is selected in the open editor is read now: the ribbon keeps the editor's focus.
		const action = editorAction(command.id, value, {
			...input,
			// Easy styles the whole box; only Expert styles the selected words (spec 09).
			selection: settings.mode === 'expert' ? getTextSelection() : null,
			// With the caret alone in the words, a bold or colour is for the words typed next.
			caret: settings.mode === 'expert' ? getTextCaret() : null,
			pending: getTextCaret() ? getPendingLook(getTextCaret()!.frame) : null,
			// Paragraph controls act on the paragraphs the caret is in, in Easy as well as Expert.
			paragraphs: getTextParagraphs(),
		});
		if (!action) return false;
		perform(action);
		// A list in the ribbon keeps the focus once it is chosen from; the words being edited get it back.
		if (
			document.activeElement instanceof Element &&
			document.activeElement.closest('[data-ribbon]')
		)
			openEditor()?.focus();
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
					{find.open && <FindBar find={{ ...find, close: closeFind }} expert={expert} />}
					{notice && (
						<div className={styles.masterBanner} role="status">
							<span>{notice}</span>
							<button type="button" onClick={() => setNotice(null)}>
								{t('templates.dismiss')}
							</button>
						</div>
					)}
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
							book={drawn}
							spread={drawnSpread ?? spread}
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
							onSelect={(ids) => {
								const target = linking !== null && ids.length === 1 ? ids[0] : undefined;
								const frame = book.spreads.flatMap((s) => s.frames).find((f) => f.id === target);
								// Choosing the frame the words continue in chains it after the one waiting.
								if (frame?.type === 'text' && !frame.flow && frame.id !== linking) {
									apply({ type: 'LinkFrame', frame: frame.id, to: linking! });
									setLinking(null);
									select([frame.id]);
								} else if (linking === null) {
									select(ids);
								}
							}}
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
							onActivate={(frame) => {
								// What Folio draws on the cover, the barcode, is made in the barcode dialog.
								const generated = ['cover_bar', 'cover_isbn', 'cover_addon'];
								if (!generated.some((p) => frame.startsWith(p)) || spread?.kind !== 'cover')
									return false;
								setBarcodeOpen(true);
								return true;
							}}
							hint={barcodeHint}
							onEditText={(frame, text) => keepAsStep({ type: 'SetText', frame, text })}
							newEditor={settings.editing.newEditor}
							onEditRequest={textEdits.onEdit}
							onEditUndo={() => store.step('undo')}
							onEditRedo={() => store.step('redo')}
							onPasteHtml={(html) => client.pasteHtml(id, html).then((read) => read.paragraphs)}
							onEditTable={(frame, edit) => keepAsStep({ type: 'EditTable', frame, edit })}
							onEditStory={(flow, paragraph) => {
								setStoryPara(paragraph ?? null);
								setStoryFlow(flow);
							}}
							onEditParagraphs={(flow, from, before, after) => {
								const styles = (book.flows.find((f) => f.id === flow)?.paragraphs ?? [])
									.slice(from, from + before.length)
									.map((p) => p.style ?? null);
								for (const intent of rangeIntents(flow, from, before, after, styles))
									keepAsStep(intent);
							}}
							speller={speller}
							find={
								find.open
									? {
											matches: marked,
											current: find.index === null ? null : (find.matches[find.index] ?? null),
											focus: find.focus,
										}
									: undefined
							}
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
								if (command === 'paste') clipboard.paste({ kind: 'point', ...point });
								else if (kind) addFrame(kind, point);
								else if (command === 'pageBackground')
									setBackground({ page: pageAt(book, spread, point) });
								else if (command.startsWith('editMaster:'))
									setEditing(command.slice('editMaster:'.length));
								else if (command === 'toggleMargins') setMarks({ margins: !marks.margins });
								else if (command === 'toggleGrid') {
									const on = !(marks.columns || marks.baseline);
									setMarks({ columns: on, baseline: on });
								} else if (master) return;
								else if (command === 'addTemplate') setTemplatesOpen(true);
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
								if (GROUP_COMMANDS.includes(command)) {
									const frames = ids.flatMap((id) => spread.frames.filter((f) => f.id === id));
									if (command === 'group') makeGroup(frames);
									else if (frames[0]) breakGroup(frames[0]);
									return;
								}
								if (CLIPBOARD_COMMANDS.includes(command)) {
									if (command === 'paste') clipboard.paste();
									else if (command === 'copy') void clipboard.copy();
									else void clipboard.cut();
									return;
								}
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
							book={drawn}
							revision={copy.revision}
							device={preview}
							spread={drawnSpread ?? spread}
							layout={layout}
							images={images}
						/>
					)}
				</div>
				<Sidebar
					group={group === 'swatches' && settings.mode !== 'expert' ? 'properties' : group}
					onGroup={setGroup}
					groups={settings.mode === 'expert' ? EXPERT_GROUPS : undefined}
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
								paragraphs={textParagraphs}
								paints={paints}
								images={images}
								choosePicture={(frame) => void choosePicture(frame)}
								languages={languages}
								linking={linking}
								onLink={setLinking}
								onEditStory={(flow) => {
									setStoryPara(null);
									setStoryFlow(flow);
								}}
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
						swatches: <SwatchesPanel book={book} apply={apply} />,
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
								chapters={chapters}
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
			<div className={rowStyles.row}>
				<div className={rowStyles.slot}>
					<Shelf
						book={drawn}
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
				</div>
				{jaxOpen && settings.jax !== 'off' && (
					<JaxPanel
						context={{
							picture: selected.some((f) => f.type === 'image'),
							text: selected.some((f) => f.type === 'text' || f.type === 'shape'),
							reader: preview,
							flowing: book.layout === 'flowing',
							grid: marks.columns || marks.baseline,
						}}
						issues={layout?.preflight ?? []}
						spreadName={labels[shown] ?? ''}
						guided={settings.jax === 'guide'}
						onHush={() => setJaxOpen(false)}
						onShow={(action) => {
							switch (action.kind) {
								case 'page':
									setEditing(null);
									setSpreadIndex(Math.min(action.index, book.spreads.length - 1));
									break;
								case 'preview':
									setPreview(action.mode);
									break;
								case 'check':
									setChecking(true);
									break;
								case 'export':
									setExporting({ quick: false });
									break;
								case 'details':
									setDetailsOpen(true);
									break;
								case 'coverSheet':
									setSheetOpen(true);
									break;
								default:
									break;
							}
						}}
					/>
				)}
			</div>
			<ExportDialog
				open={exporting !== null && !dialogWindows}
				onClose={() => setExporting(null)}
				client={client}
				id={id}
				autoStart={exporting?.quick ? ['sharingPdf'] : null}
				onCheck={() => {
					setExporting(null);
					setChecking(true);
				}}
			/>
			<ImportReportDialog
				outcome={dialogWindows ? null : imported}
				onClose={() => setImported(null)}
				onRemap={(mapping) => {
					if (importedPath.current !== null) void bringIn(importedPath.current, mapping);
				}}
			/>
			<StoryEditorDialog
				open={showStory}
				onClose={() => setStoryFlow(null)}
				book={book}
				flow={storyFlow}
				focusParagraph={storyPara}
				speller={speller}
				choices={choices}
				apply={apply}
			/>
			<BookDetailsDialog
				open={showDetails}
				onClose={() => setDetailsOpen(false)}
				book={book}
				onSave={(details) => apply({ type: 'SetDetails', details })}
				onPaper={(paper) => apply({ type: 'SetPaper', paper })}
			/>
			<SpreadTemplatesDialog
				open={showTemplates}
				onClose={() => setTemplatesOpen(false)}
				client={client}
				id={id}
				onChoose={async (template, name) => {
					await addTemplate(template, name);
					setTemplatesOpen(false);
				}}
			/>
			<LinkDialog
				open={linkDialog !== null}
				onClose={() => setLinkDialog(null)}
				words={linkDialog?.words ?? ''}
				href={linkDialog?.target.href ?? ''}
				editing={linkDialog?.target.link ?? false}
				mixed={linkDialog?.target.mixed ?? false}
				hasWords={linkDialog?.target.words ?? false}
				spreads={labels}
				onSet={(href) => {
					if (linkDialog) apply(linkIntent(linkDialog.scope, linkDialog.target, href));
				}}
				onRemove={() => {
					if (linkDialog) apply(linkIntent(linkDialog.scope, linkDialog.target, ''));
				}}
			/>
			<BarcodeDialog
				open={showBarcode}
				onClose={() => setBarcodeOpen(false)}
				book={book}
				apply={apply}
			/>
			<BookSizeDialog
				open={showBookSize}
				onClose={() => setBookSizeOpen(false)}
				book={book}
				units={settings.units}
				apply={apply}
			/>
			<MarginsDialog
				open={showMargins}
				onClose={() => setMarginsOpen(false)}
				book={book}
				units={settings.units}
				apply={apply}
			/>
			<CoverSheetDialog
				open={showSheet}
				onClose={() => setSheetOpen(false)}
				client={client}
				id={id}
				revision={copy.revision}
				book={book}
				onSave={(details) => apply({ type: 'SetDetails', details })}
				onPutDescription={(text) => {
					const intent = putDescriptionIntent(book, text);
					if (intent) apply(intent);
				}}
			/>
			<SoftProofDialog
				open={showProof}
				onClose={() => setProofOpen(false)}
				client={client}
				id={id}
				spread={spread.id}
			/>
			<CheckBookDialog
				open={showCheck}
				onClose={() => setChecking(false)}
				client={client}
				id={id}
				book={book}
				labels={labels}
				onIgnore={(keys) => void store.apply({ type: 'SetPreflightIgnore', keys })}
				onShow={showIssue}
			/>
			<PageBackgroundDialog
				open={showBackground}
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
