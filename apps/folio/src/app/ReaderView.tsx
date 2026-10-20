// The reader preview: the book as a Kindle or a Kobo would show it, in a frame of that device
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { ReaderPreview } from '@liminal-hq/folio-protocol/generated/ReaderPreview';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react';
import type { BooksClient } from '../books/booksClient';
import { SpreadCanvas } from '../canvas/SpreadCanvas';
import { t } from '../i18n/messages';
import {
	DEVICES,
	fontFaceRules,
	pageCount,
	readerDocument,
	TEXT_STEPS,
	THEMES,
} from './readerDocument';
import type { ReaderDevice, ReaderTheme } from './readerDocument';
import styles from './ReaderView.module.css';

/** The scale that fits a box of `width` × `height` in what is left of `room` once `reserve` is spent. */
function useFit(width: number, height: number, reserve: number) {
	const room = useRef<HTMLElement>(null);
	const [scale, setScale] = useState(1);
	useEffect(() => {
		const element = room.current;
		if (!element) return;
		const measure = () => {
			const free = element.clientHeight - reserve;
			setScale(free > 0 ? Math.min(1, free / height, element.clientWidth / width) : 1);
		};
		measure();
		if (typeof ResizeObserver === 'undefined') return;
		const watch = new ResizeObserver(measure);
		watch.observe(element);
		return () => watch.disconnect();
	}, [width, height, reserve]);
	return { room, scale };
}

interface ReaderViewProps {
	client: BooksClient;
	id: string;
	book: Book;
	/** Changes when the book does, so the preview is made again. */
	revision: number;
	device: ReaderDevice;
	/** The spread being looked at, for a book of fixed pages. */
	spread: Spread | null;
	layout: SpreadLayout | null;
	images: Record<string, string>;
}

export function ReaderView({
	client,
	id,
	book,
	revision,
	device,
	spread,
	layout,
	images,
}: ReaderViewProps) {
	const flowing = book.layout === 'flowing';
	const spec = DEVICES[device];
	// The device is shown whole, scaled down when the stage is not as tall as a Paperwhite.
	const shown = flowing
		? { w: spec.width + 40, h: spec.height + 40 }
		: { w: spec.height + 40, h: spec.width + 40 };
	const { room, scale } = useFit(shown.w, shown.h, 110);
	return (
		<section
			ref={room}
			className={styles.view}
			style={{ '--folio-reader-scale': scale } as CSSProperties}
			aria-label={t('reader.label', { device: t(DEVICES[device].name) })}
		>
			{flowing ? (
				<Reflowed client={client} id={id} revision={revision} device={device} />
			) : (
				<Fixed book={book} device={device} spread={spread} layout={layout} images={images} />
			)}
			<p className={styles.note}>{t(flowing ? 'reader.note.reflow' : 'reader.note.fixed')}</p>
		</section>
	);
}

function Reflowed({
	client,
	id,
	revision,
	device,
}: {
	client: BooksClient;
	id: string;
	revision: number;
	device: ReaderDevice;
}) {
	const [preview, setPreview] = useState<ReaderPreview | null>(null);
	const [failed, setFailed] = useState(false);
	const [step, setStep] = useState(TEXT_STEPS.indexOf(1));
	const [theme, setTheme] = useState<ReaderTheme>('white');
	const [readerFont, setReaderFont] = useState(false);
	const [page, setPage] = useState(0);
	const [pages, setPages] = useState(1);
	const frame = useRef<HTMLIFrameElement>(null);
	const spec = DEVICES[device];

	useEffect(() => {
		let live = true;
		setFailed(false);
		client
			.readerPreview(id)
			.then((found) => live && setPreview(found))
			.catch(() => live && setFailed(true));
		return () => {
			live = false;
		};
	}, [client, id, revision]);

	// The fonts the page already has are lent to the preview, which is a document of its own.
	const faces = useMemo(() => fontFaceRules(document), []);
	const srcDoc = useMemo(
		() =>
			preview
				? readerDocument({
						preview,
						faces,
						scale: TEXT_STEPS[step] ?? 1,
						theme,
						readerFont,
					})
				: '',
		[preview, faces, step, theme, readerFont],
	);

	// The page being shown, for a document that finishes loading after the page was chosen.
	const current = useRef(0);
	current.current = page;
	const measure = () => {
		const doc = frame.current?.contentDocument;
		if (!doc) return;
		const total = pageCount(doc.documentElement.scrollWidth, spec.width);
		setPages(total);
		const kept = Math.min(current.current, total - 1);
		setPage(kept);
		doc.documentElement.scrollLeft = kept * spec.width;
	};

	useEffect(() => {
		const doc = frame.current?.contentDocument;
		if (doc) doc.documentElement.scrollLeft = page * spec.width;
	}, [page, spec.width, srcDoc]);

	return (
		<>
			<div className={styles.bar} role="toolbar" aria-label={t('reader.controls')}>
				<button type="button" onClick={() => setPage(Math.max(0, page - 1))} disabled={page === 0}>
					{t('reader.previous')}
				</button>
				<span role="status" className={styles.where}>
					{t('reader.page', { page: page + 1, pages })}
				</span>
				<button
					type="button"
					onClick={() => setPage(Math.min(pages - 1, page + 1))}
					disabled={page >= pages - 1}
				>
					{t('reader.next')}
				</button>
				<span className={styles.gap} />
				<button
					type="button"
					onClick={() => setStep(Math.max(0, step - 1))}
					disabled={step === 0}
					aria-label={t('reader.smaller')}
				>
					A−
				</button>
				<button
					type="button"
					onClick={() => setStep(Math.min(TEXT_STEPS.length - 1, step + 1))}
					disabled={step === TEXT_STEPS.length - 1}
					aria-label={t('reader.bigger')}
				>
					A+
				</button>
				<select
					aria-label={t('reader.theme')}
					value={theme}
					onChange={(event) => setTheme(event.target.value as ReaderTheme)}
				>
					{THEMES.map((name) => (
						<option key={name} value={name}>
							{t(`reader.theme.${name}`)}
						</option>
					))}
				</select>
				<label className={styles.check}>
					<input
						type="checkbox"
						checked={readerFont}
						onChange={(event) => setReaderFont(event.target.checked)}
					/>
					{t('reader.readerFont')}
				</label>
			</div>
			<div
				className={styles.fitter}
				style={
					{
						'--folio-fit-w': `${spec.width + 40}px`,
						'--folio-fit-h': `${spec.height + 40}px`,
					} as CSSProperties
				}
			>
				<div className={styles.device} style={{ width: spec.width + 40, height: spec.height + 40 }}>
					{failed && (
						<p role="alert" className={styles.problem}>
							{t('reader.failed')}
						</p>
					)}
					{preview && (
						<iframe
							ref={frame}
							className={styles.screen}
							title={t('reader.screen')}
							sandbox="allow-same-origin"
							srcDoc={srcDoc}
							width={spec.width}
							height={spec.height}
							onLoad={measure}
						/>
					)}
				</div>
			</div>
		</>
	);
}

function Fixed({
	book,
	device,
	spread,
	layout,
	images,
}: {
	book: Book;
	device: ReaderDevice;
	spread: Spread | null;
	layout: SpreadLayout | null;
	images: Record<string, string>;
}) {
	const spec = DEVICES[device];
	// A fixed book is shown a spread at a time on a screen turned on its side.
	const width = spec.height;
	const height = spec.width;
	const pointsWide = book.trim.w * (spread?.pages.length ?? 1);
	const zoom = Math.min((width - 40) / pointsWide, (height - 40) / book.trim.h) / (96 / 72);
	return (
		<div
			className={styles.fitter}
			style={
				{
					'--folio-fit-w': `${width + 40}px`,
					'--folio-fit-h': `${height + 40}px`,
				} as CSSProperties
			}
		>
			<div className={styles.device} style={{ width: width + 40, height: height + 40 }}>
				<div className={styles.fixed} style={{ width, height }}>
					{spread && (
						<SpreadCanvas
							book={book}
							spread={spread}
							layout={layout}
							zoom={zoom}
							images={images}
							label={t('reader.screen')}
						/>
					)}
				</div>
			</div>
		</div>
	);
}
