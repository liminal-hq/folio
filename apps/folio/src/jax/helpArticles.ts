// Jax's help library for "Ask": short task articles, found by the words in a question
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// The answer is extractive (spec 07, tier A): the best article is shown as it is written, with a
// "Show me" button that goes to the thing it talks about. It is fast, runs on this computer and
// cannot make anything up.

import type { GuideAction } from './jaxTips';

export interface HelpArticle {
	id: string;
	title: string;
	/** Words a question might use for this article. */
	keywords: readonly string[];
	/** What Jax says, in a few sentences. */
	body: string;
	/** What "Show me" does, if there is something to go to. */
	action: GuideAction;
}

export const HELP_ARTICLES: readonly HelpArticle[] = [
	{
		id: 'bleed',
		title: 'What is bleed?',
		keywords: ['bleed', 'trim', 'edge', 'edges', 'cut', 'margin', 'margins', 'white'],
		body: 'Printers trim pages after printing, and the blade drifts a little. Bleed is picture or colour that runs past the trim edge so no thin white line shows. Folio draws the bleed line on the page when Page helpers are on, and Check book tells you when a picture falls short of it.',
		action: { kind: 'check' },
	},
	{
		id: 'spine',
		title: 'How wide is my spine?',
		keywords: ['spine', 'cover', 'wrap', 'paperback', 'back', 'width', 'thick'],
		body: 'The spine depends on the page count and the paper. Open the cover sheet to see the back, spine and front laid out as one sheet, with the folds, safe areas and the barcode box drawn over it. Spine text needs at least 80 pages.',
		action: { kind: 'coverSheet' },
	},
	{
		id: 'page-numbers',
		title: 'How do I add page numbers?',
		keywords: ['page', 'pages', 'number', 'numbers', 'numbering', 'folio', 'footer'],
		body: 'Choose Centre or Outside under Page numbers in the Properties panel on the right. Folio puts the numbers on the master pages for you, so every page gets one and they keep counting as pages move.',
		action: { kind: 'none' },
	},
	{
		id: 'kindle-kobo',
		title: 'How will it look on a Kindle or Kobo?',
		keywords: [
			'kindle',
			'kobo',
			'ereader',
			'e-reader',
			'ebook',
			'epub',
			'reader',
			'screen',
			'preview',
		],
		body: 'Use the View tab: Kindle and Kobo show your book on a device screen, with page turns, text size steps and white, sepia or dark pages. Real devices differ a little, and Jax says so in the preview. Print takes you back to the pages.',
		action: { kind: 'preview', mode: 'kindle' },
	},
	{
		id: 'import',
		title: 'Can I bring in my manuscript?',
		keywords: [
			'import',
			'word',
			'docx',
			'odt',
			'rtf',
			'markdown',
			'text',
			'manuscript',
			'paste',
			'novel',
			'write',
		],
		body: 'Yes. File → Import text takes a Word (.docx), OpenDocument (.odt), Rich Text (.rtf), Markdown or plain text file and pours it into your book as its story, in one undo step. Headings become chapter titles, and the report says anything it left out. It works in a book with flowing text.',
		action: { kind: 'import' },
	},
	{
		id: 'amazon',
		title: 'How do I send it to Amazon KDP?',
		keywords: ['amazon', 'kdp', 'export', 'print', 'pdf', 'upload', 'publish', 'send', 'paperback'],
		body: 'Open the Export tab and choose the paperback targets for the interior and the cover. Run Check book first so nothing is a surprise. Folio writes the files into an exports folder beside your book.',
		action: { kind: 'export' },
	},
	{
		id: 'isbn',
		title: 'Where does my ISBN go?',
		keywords: ['isbn', 'barcode', 'ean', 'price', 'title', 'author', 'details'],
		body: 'Give the book its title, author and ISBN under Book details. The barcode is a choice on the Insert tab: let KDP add one, or have Folio draw your own with an optional price add-on.',
		action: { kind: 'details' },
	},
	{
		id: 'check',
		title: 'Is my book ready?',
		keywords: [
			'ready',
			'check',
			'problems',
			'errors',
			'warnings',
			'preflight',
			'wrong',
			'fix',
			'overset',
		],
		body: 'Check book lays out every page and lists what needs you, what is worth a look and what is for your information. Each item has a way to fix it and a Show me button that takes you to the page.',
		action: { kind: 'check' },
	},
	{
		id: 'colour',
		title: 'How do I get the colour right for print?',
		keywords: [
			'colour',
			'color',
			'cmyk',
			'ink',
			'swatch',
			'swatches',
			'black',
			'proof',
			'press',
			'rgb',
		],
		body: 'In Expert mode the Swatches tab lets you name a colour with exact ink percentages, and Soft proof in the Export tab shows how a press would print the page. Small text looks best in plain black, not a black made of several inks.',
		action: { kind: 'none' },
	},
	{
		id: 'undo',
		title: 'I made a mistake',
		keywords: ['undo', 'mistake', 'oops', 'back', 'redo', 'revert', 'lost', 'deleted'],
		body: 'Press Ctrl+Z to take back one step at a time, and Ctrl+Y or Ctrl+Shift+Z to put it back. Folio also keeps a safety copy while you work, so a crash will not lose your book.',
		action: { kind: 'none' },
	},
];

const WORD = /[a-z0-9]+/g;

/** The article that best fits a question, or `null` when nothing in it is a word Jax knows. */
export function askHelp(question: string): HelpArticle | null {
	const words = question.toLowerCase().match(WORD) ?? [];
	let best: HelpArticle | null = null;
	let top = 0;
	for (const article of HELP_ARTICLES) {
		let score = 0;
		for (const word of words) {
			for (const key of article.keywords) {
				if (key === word) score += 3;
				else if (word.length > 3 && (key.startsWith(word) || word.startsWith(key))) score += 1;
			}
			if (article.title.toLowerCase().includes(word) && word.length > 3) score += 1;
		}
		if (score > top) {
			top = score;
			best = article;
		}
	}
	return best;
}
