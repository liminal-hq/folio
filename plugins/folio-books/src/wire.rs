// The shapes the plugin sends to the webview, generated as TypeScript
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::wire::SpreadLayout;
use folio_model::{Book, BookKind};
use folio_session::Patch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// One open book, in a line: enough for a tab, a title bar and the recents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BookSummary {
    /// The session's id, which every other command takes. Not the book's own id: the same book
    /// opened later is a new session.
    pub id: String,
    pub title: String,
    pub kind: BookKind,
    /// Where the book was opened from or last saved to; `None` for one that has never been saved.
    pub path: Option<String>,
    /// There is something to save: a new book that was never saved, or changes since the last save.
    pub dirty: bool,
}

/// A spread laid out, and the revision of the book it was laid out from. A window draws it only
/// while that is the revision it holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct SpreadLayoutResult {
    #[ts(type = "number")]
    pub revision: u64,
    pub layout: SpreadLayout,
}

/// An open book in full, for a window that is showing it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BookSnapshot {
    pub summary: BookSummary,
    /// The revision the book is at. Events with a higher revision apply on top of this.
    #[ts(type = "number")]
    pub revision: u64,
    pub book: Book,
    /// `styles.json`, the named styles and the look.
    #[ts(type = "unknown")]
    pub styles: Value,
    /// `settings.json`, the book's own choices such as the paper.
    #[ts(type = "unknown")]
    pub settings: Value,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Sent to every window after a change to an open book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PatchEvent {
    pub id: String,
    pub patch: Patch,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Sent to every window when books open, close, are saved, or become changed or clean.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BooksChanged {
    pub books: Vec<BookSummary>,
}

/// A word the speller does not know, by where it is in the text asked about, in UTF-16 code units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Misspelling {
    pub start: u32,
    pub end: u32,
    pub word: String,
}

/// What checking a text found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct SpellCheck {
    /// Whether there is a dictionary for the book's language. Without one nothing is flagged, and
    /// the UI can say so instead of showing a clean page.
    pub supported: bool,
    /// The dictionary that checked the text, by id.
    pub dictionary: Option<String>,
    /// A dictionary that would check the book's language better, which can be downloaded.
    pub offer: Option<DictionaryOffer>,
    pub misspellings: Vec<Misspelling>,
}

/// A dictionary on offer for a book's language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct DictionaryOffer {
    pub id: String,
    pub name: String,
    /// What it takes to download, in bytes.
    pub bytes: u32,
}

/// A dictionary, and whether it is in place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct DictionaryStatus {
    pub id: String,
    pub name: String,
    /// The languages it checks, as BCP 47 codes.
    pub languages: Vec<String>,
    /// Its own licence, whose text is kept beside it.
    pub licence: String,
    pub bytes: u32,
    pub installed: bool,
    /// It comes with Folio and cannot be removed.
    pub bundled: bool,
}

/// Where a font a book can use comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum FontOrigin {
    /// Ships with Folio.
    Bundled,
    /// Kept inside the book.
    Book,
    /// Installed on this computer.
    System,
}

/// One typeface family a book can draw in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct FontChoice {
    pub family: String,
    pub origin: FontOrigin,
}

/// What a font's licence lets Folio do with its file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum FontLicence {
    /// The file may be kept in a book, put in an EPUB and embedded in a PDF.
    Free,
    /// It may be embedded in a PDF but its file may not be kept in a book or an EPUB.
    PrintOnly,
    /// It may not be embedded anywhere.
    Restricted,
}

/// A typeface a book uses, and what is known of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct FontUse {
    pub family: String,
    pub origin: FontOrigin,
    /// No font of that name is bundled, kept or installed, so a stand-in is used.
    pub missing: bool,
    pub licence: FontLicence,
}

/// Why a font was not kept with a book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum FontSkip {
    /// The font's licence does not let its file be kept in a book.
    NotAllowed,
    /// The file is too big to keep.
    TooLarge,
    /// No installed font has that family.
    NotInstalled,
}

/// A family `keep_fonts` could not keep, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct SkippedFont {
    pub family: String,
    pub reason: FontSkip,
}

/// What keeping fonts with a book did.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct KeepFontsReport {
    /// The families now kept with the book, including any that already were.
    pub kept: Vec<String>,
    pub skipped: Vec<SkippedFont>,
}

/// A kind of file to make from a book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ExportTarget {
    /// A PDF to read on a screen or send round.
    SharingPdf,
    /// The interior of a paperback, as Kindle Direct Publishing asks for it.
    KdpInterior,
    /// Every page for a print shop: bleed all round, crop marks, PDF/X-4.
    PrintShopPdf,
    /// The paperback cover for KDP: back, spine and front on one sheet.
    KdpCover,
    /// The cover as a picture for the book's page in a store.
    EbookCover,
    /// A reflowable EPUB for Kindle Direct Publishing.
    KindleEpub,
    /// A fixed-layout EPUB: every page as laid out, for a picture book.
    FixedEpub,
    /// A reflowable EPUB for Kobo, and its Kobo flavour (KEPUB).
    KoboEpub,
}

/// A file an export wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ExportedFile {
    pub target: ExportTarget,
    pub path: String,
    pub bytes: u32,
}

/// How print files hold colour: sRGB as drawn, or as ink.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ExportColour {
    /// Write the KDP interior in CMYK ink rather than sRGB. The KDP cover is always in ink.
    pub cmyk: bool,
    /// The path of the press's CMYK ICC profile; without one, ink is a plain separation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub profile: Option<String>,
}

/// One page of a spread as drawn and as a press would print it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ProofPage {
    /// The page as drawn, a `data:image/png` URI.
    pub original: String,
    /// The page through the press's profile, a `data:image/png` URI.
    pub proofed: String,
    /// How much of the page, in per cent, is a colour the press cannot reproduce.
    pub outside: f64,
}

/// A swatch of the book as drawn and as a press would print it, each as `#rrggbb`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ProofColour {
    pub name: String,
    pub display: String,
    pub proofed: String,
    /// The press cannot reproduce it.
    pub outside: bool,
}

/// A spread's pages through a press, and the book's swatches through it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ProofSpread {
    pub pages: Vec<ProofPage>,
    pub colours: Vec<ProofColour>,
}

/// The paperback cover sheet drawn, and where its parts are, in points from the sheet's top-left
/// corner (the bleed included).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct CoverSheetPreview {
    /// The sheet as a `data:image/png` URI.
    pub image: String,
    /// The whole sheet, bleed included.
    pub width: f64,
    pub height: f64,
    /// The bleed all round.
    pub edge: f64,
    /// The width of one cover (the trim width) and of the spine.
    pub cover: f64,
    pub spine: f64,
    /// Whether the spine is thick enough for text.
    pub spine_text: bool,
    /// The box KDP keeps for its barcode.
    pub barcode: folio_model::Rect,
}

/// What an export wrote, and where.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ExportReport {
    /// The folder the files are in.
    pub folder: String,
    pub files: Vec<ExportedFile>,
    /// Set when the person stopped the export, so the files are those made before that.
    #[serde(default)]
    pub cancelled: bool,
}

/// A chapter of the book as a reader shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PreviewChapter {
    pub title: String,
    /// The chapter's XHTML body. Pictures are `../images/<name>`.
    pub body: String,
}

/// A picture of the book, ready to put in a page as an address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PreviewImage {
    pub name: String,
    pub data_uri: String,
}

/// The book as an e-reader would show it: the EPUB's own chapters and CSS (D-35).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ReaderPreview {
    pub title: String,
    pub language: String,
    pub css: String,
    pub chapters: Vec<PreviewChapter>,
    pub images: Vec<PreviewImage>,
}

/// A book with changes that were never saved, found after Folio stopped without closing it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Recoverable {
    pub id: String,
    pub title: String,
    /// Where the real file is, or nothing for a book that was never saved.
    pub path: Option<String>,
    /// When the copy was written, in seconds since 1970.
    pub saved_at: u32,
}

/// How far a dictionary download has got, sent to every window as it goes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct DictionaryProgress {
    pub id: String,
    /// Bytes that have arrived, over every file of the dictionary.
    pub received: u32,
    /// Bytes expected in all: the sizes the catalogue pins.
    pub total: u32,
}

/// How far an export has got, sent to every window as it goes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ExportProgress {
    /// The book being exported.
    pub id: String,
    /// The target about to be made, and how many are done before it.
    pub target: ExportTarget,
    pub done: u32,
    pub total: u32,
}

/// A misspelt word somewhere in the book, and where to find it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct MisspeltWord {
    pub spread: String,
    /// The frame it is in; for flowing text, the first frame of the flow.
    pub frame: String,
    /// The row and column of the table cell it is in, for a table.
    pub row: Option<u32>,
    pub col: Option<u32>,
    pub word: String,
    /// Where it is in the frame's (or cell's) words, in UTF-16 code units.
    pub start: u32,
    pub end: u32,
    /// A few words round it, to know it by.
    pub context: String,
    /// Whether it can be changed from here: its words are plain text of a frame of its own.
    /// Flowing text and words with emphasis are changed on the page.
    pub replaceable: bool,
}

/// Every misspelt word in a book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BookSpelling {
    pub supported: bool,
    pub dictionary: Option<String>,
    pub offer: Option<DictionaryOffer>,
    pub words: Vec<MisspeltWord>,
    /// There were more than are listed.
    pub truncated: bool,
}
