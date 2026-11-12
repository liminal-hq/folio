// The structure of a PDF's pages: what is content, what is decoration, and in what order to read it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A tagged PDF says what its marks are. `plan` works that out from the page scenes a file is
//! about to draw and the book they came from, and touches no PDF: it answers, for every item of
//! every page, whether it is an artifact (decoration a screen reader skips) or content with a
//! place in the reading order, and gives the structure tree (paragraphs, headings, figures,
//! tables, links) that the content belongs to. The PDF writer draws with those answers and hands
//! the tree to the document.
//!
//! **Reading order.** Sheets in the order of the file, pages left to right on a sheet. On a page
//! the content frames are read in rows, top to bottom (frames whose heights overlap by more than
//! half the shorter are one row), and left to right inside a row, with frames at about the same
//! left edge read top to bottom and the stacking order breaking any tie. A story that runs through
//! several frames is read as one thread, in the order of its frames, at the place where its first
//! frame on the page falls. The lines of one paragraph make one element however many frames or
//! pages they are on. Words that are all in one title frame make one heading.
//!
//! **Decoration** (artifacts): the backgrounds, everything that comes from a master page, page
//! numbers, running heads, the spine and ornament styles, table tints and rules, a shape's art
//! unless it has a description, a picture frame still waiting for its picture, the frames
//! `folio_layout::decoration` finds (marked decorative, page numbers and their backdrops), and the
//! copy of a frame that crosses the gutter on the page that does not hold its middle.

use std::collections::{BTreeSet, HashMap};
use std::ops::Range;

use folio_layout::decoration::{is_page_number_text, Decorations};
use folio_layout::preflight::page_rects;
use folio_layout::scene::{PageScene, SceneKind};
use folio_layout::wire::{LayoutLine, LayoutRun};
use folio_model::{Book, Frame};

use crate::roles::{printed_of, role_of, Printed, Role};

/// A mark the PDF writer will make, found by `LeafId` in `Plan::sites`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LeafId(pub usize);

/// Why something is not content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    PageNumber,
    Header,
    Footer,
    /// Furniture of the layout: ornaments, tints, rules, decoration, the copy of a frame that is
    /// read on the other page.
    Layout,
    /// What fills the page behind everything.
    Background,
}

/// An item of a page scene, by its place in the file: the sheet, the page on it (left to right)
/// and the item in `PageScene::items`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemAt {
    pub sheet: usize,
    pub member: usize,
    pub item: usize,
}

/// What to do with one item when it is drawn.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemPlan {
    /// The whole item is decoration.
    Artifact(ArtifactKind),
    /// A picture or a shape's art that is content: one mark for the whole drawing.
    Figure(LeafId),
    /// Words, line by line.
    Text(Vec<LinePlan>),
}

/// What to do with one line of words.
#[derive(Debug, Clone, PartialEq)]
pub enum LinePlan {
    Artifact(ArtifactKind),
    /// The runs of the line, cut into pieces that each get a mark.
    Pieces(Vec<Piece>),
}

/// Some runs of a line that are marked together: they are in one language and link to one place.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub runs: Range<usize>,
    pub leaf: LeafId,
    /// The language of the words when it is not the book's.
    pub lang: Option<String>,
    /// Where the words link to, when they do.
    pub href: Option<String>,
}

/// Where a mark is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct LeafSite {
    pub at: ItemAt,
    /// The line of the item, for words; `None` for a figure.
    pub line: Option<usize>,
    /// The runs of the line the mark covers; empty for a figure.
    pub runs: Range<usize>,
}

/// What an element of the tree is.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Paragraph,
    /// A heading of `level` (1 to 6), with the words it says.
    Heading {
        level: u8,
        title: String,
    },
    BlockQuote,
    Caption,
    Note,
    /// A picture, with its description when it has one.
    Figure {
        alt: Option<String>,
    },
    /// Words that link to `href`. `id` is the first mark inside, which names the element so the
    /// writer can add the link annotations of those words to it.
    Link {
        href: String,
        id: LeafId,
    },
    Table,
    TableHead,
    TableBody,
    Row,
    /// A header cell.
    HeaderCell,
    Cell,
}

/// An element of the structure tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub kind: Kind,
    pub children: Vec<Node>,
}

/// A child in the tree: a mark, or an element of its own.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Leaf(LeafId),
    Group(Group),
}

/// Everything the writer needs to tag a file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Plan {
    pub items: HashMap<ItemAt, ItemPlan>,
    pub sites: Vec<LeafSite>,
    /// The elements of the document in reading order.
    pub tree: Vec<Node>,
}

impl Plan {
    /// For each mark inside a link, the link it is in.
    pub fn link_of_leaf(&self) -> HashMap<LeafId, LeafId> {
        fn walk(nodes: &[Node], out: &mut HashMap<LeafId, LeafId>) {
            for node in nodes {
                if let Node::Group(group) = node {
                    if let Kind::Link { id, .. } = &group.kind {
                        for child in &group.children {
                            if let Node::Leaf(leaf) = child {
                                out.insert(*leaf, *id);
                            }
                        }
                    }
                    walk(&group.children, out);
                }
            }
        }
        let mut out = HashMap::new();
        walk(&self.tree, &mut out);
        out
    }

    /// Every mark in the tree, in reading order.
    pub fn leaves_in_order(&self) -> Vec<LeafId> {
        fn walk(nodes: &[Node], out: &mut Vec<LeafId>) {
            for node in nodes {
                match node {
                    Node::Leaf(leaf) => out.push(*leaf),
                    Node::Group(group) => walk(&group.children, out),
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.tree, &mut out);
        out
    }
}

/// The plan for the `sheets` of a file, each the pages on it left to right, drawn from `book`.
pub fn plan(book: &Book, sheets: &[Vec<&PageScene>]) -> Plan {
    let sheets: Vec<Vec<(usize, &PageScene)>> = sheets
        .iter()
        .map(|s| s.iter().map(|p| (0, *p)).collect())
        .collect();
    plan_books(&[book], &sheets)
}

/// The plan for a file whose pages come from more than one book (the back cover is a view of the
/// book cut down to its back panel): each page is paired with the index in `books` of the book it
/// was laid out from.
pub fn plan_books(books: &[&Book], sheets: &[Vec<(usize, &PageScene)>]) -> Plan {
    Planner::new(books, sheets).run()
}

/// How many figures in `plan` have no description.
pub fn undescribed_figures(plan: &Plan) -> usize {
    fn count(nodes: &[Node]) -> usize {
        nodes
            .iter()
            .map(|node| match node {
                Node::Leaf(_) => 0,
                Node::Group(group) => {
                    usize::from(matches!(group.kind, Kind::Figure { alt: None }))
                        + count(&group.children)
                }
            })
            .sum()
    }
    count(&plan.tree)
}

/// Whether a style's words are read as a heading, and which level.
fn heading_level(role: Role) -> Option<u8> {
    match role {
        Role::Title => Some(1),
        Role::Subheading => Some(2),
        _ => None,
    }
}

/// A frame seen on a page, with its items.
struct Entry<'a> {
    base: String,
    /// The art item (a picture, shape or table), if any.
    art: Option<usize>,
    /// The text items: one for a text frame or a shape, one for each cell of a table.
    texts: Vec<usize>,
    frame: Option<&'a Frame>,
    /// The frame's turned box on the page: left, top, width, height.
    bounds: (f64, f64, f64, f64),
    /// The first item, which is its place in the stacking order.
    z: usize,
    class: Class,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Class {
    Artifact(ArtifactKind),
    Content,
}

/// What the planner looks up in one book.
struct BookIndex<'a> {
    book: &'a Book,
    decorations: Decorations,
    /// A page's id to its spread and its place in it.
    pages: HashMap<&'a str, (usize, usize)>,
    /// For a frame, the pages of its spread it is drawn on in the file.
    drawn_on: HashMap<(usize, String), BTreeSet<usize>>,
}

struct Planner<'a> {
    books: Vec<BookIndex<'a>>,
    /// The book the page being planned was laid out from.
    cur: usize,
    sheets: &'a [Vec<(usize, &'a PageScene)>],
    plan: Plan,
}

impl<'a> Planner<'a> {
    fn new(books: &[&'a Book], sheets: &'a [Vec<(usize, &'a PageScene)>]) -> Self {
        let books = books
            .iter()
            .enumerate()
            .map(|(at, &book)| {
                let mut pages = HashMap::new();
                for (s, spread) in book.spreads.iter().enumerate() {
                    for (p, page) in spread.pages.iter().enumerate() {
                        pages.insert(page.id.0.as_str(), (s, p));
                    }
                }
                let mut drawn_on: HashMap<(usize, String), BTreeSet<usize>> = HashMap::new();
                for (_, page) in sheets.iter().flatten().filter(|(b, _)| *b == at) {
                    if let Some(&(s, p)) = pages.get(page.page.0.as_str()) {
                        for item in &page.items {
                            drawn_on
                                .entry((s, base_of(&item.frame.0).to_owned()))
                                .or_default()
                                .insert(p);
                        }
                    }
                }
                BookIndex {
                    book,
                    decorations: Decorations::of_book(book),
                    pages,
                    drawn_on,
                }
            })
            .collect();
        Planner {
            books,
            cur: 0,
            sheets,
            plan: Plan::default(),
        }
    }

    fn b(&self) -> &BookIndex<'a> {
        &self.books[self.cur]
    }

    fn book(&self) -> &'a Book {
        self.books[self.cur].book
    }

    fn run(mut self) -> Plan {
        let mut out = Emit::default();
        let sheets = self.sheets;
        for (sheet, members) in sheets.iter().enumerate() {
            for (member, (book, page)) in members.iter().enumerate() {
                self.cur = *book;
                self.page(sheet, member, page, &mut out);
            }
        }
        self.plan.tree = out.finish();
        self.plan
    }

    fn leaf(&mut self, at: ItemAt, line: Option<usize>, runs: Range<usize>) -> LeafId {
        self.plan.sites.push(LeafSite { at, line, runs });
        LeafId(self.plan.sites.len() - 1)
    }

    /// What a frame is: decoration, or content.
    fn classify(&self, spread: usize, page: usize, base: &str, frame: Option<&Frame>) -> Class {
        if base.contains('@') {
            return Class::Artifact(self.master_kind(base));
        }
        let spread_id = &self.book().spreads[spread].id;
        if let Some(frame) = frame {
            if let Frame::Text(text) = frame {
                if let Some(printed) = printed_of(&text.style.0) {
                    return Class::Artifact(self.furniture_kind(printed, &frame.base().rect));
                }
            }
            if let Some(why) = self.b().decorations.of(spread_id, &frame.base().id) {
                use folio_layout::decoration::Decoration;
                return Class::Artifact(match why {
                    Decoration::PageNumber => ArtifactKind::PageNumber,
                    Decoration::Marked if text_is_a_number(frame) => ArtifactKind::PageNumber,
                    _ => ArtifactKind::Layout,
                });
            }
        }
        // A frame that crosses the gutter is drawn on both pages and is read on the one that holds
        // its middle (or on the one there is, when the other is not in the file).
        if let (Some(frame), Some(pages)) =
            (frame, self.b().drawn_on.get(&(spread, base.to_owned())))
        {
            if pages.len() > 1 && page != self.owner(spread, frame, pages) {
                return Class::Artifact(ArtifactKind::Layout);
            }
        }
        Class::Content
    }

    /// The page, of those `pages` a frame is drawn on, that reads it.
    fn owner(&self, spread: usize, frame: &Frame, pages: &BTreeSet<usize>) -> usize {
        let layout = page_rects(self.book(), &self.book().spreads[spread]);
        let base = frame.base();
        let bounds = base.rect.turned_bounds(base.rotation);
        let centre = bounds.x.0 + bounds.w.0 / 2.0;
        let holding = layout
            .iter()
            .position(|p| centre >= p.rect.x.0 && centre < p.rect.x.0 + p.rect.w.0);
        match holding {
            Some(p) if pages.contains(&p) => p,
            _ => *pages.iter().next().expect("drawn somewhere"),
        }
    }

    fn furniture_kind(&self, printed: Printed, rect: &folio_model::Rect) -> ArtifactKind {
        match printed {
            Printed::PageNumber => ArtifactKind::PageNumber,
            Printed::RunningHead => {
                let middle = rect.y.0 + rect.h.0 / 2.0;
                if middle < self.book().trim.h.0 / 2.0 {
                    ArtifactKind::Header
                } else {
                    ArtifactKind::Footer
                }
            }
            Printed::Spine | Printed::Ornament => ArtifactKind::Layout,
        }
    }

    /// Everything from a master page is furniture; which kind follows its style.
    fn master_kind(&self, base: &str) -> ArtifactKind {
        let Some((frame_id, page_id)) = base.split_once('@') else {
            return ArtifactKind::Layout;
        };
        let Some(&(s, p)) = self.b().pages.get(page_id) else {
            return ArtifactKind::Layout;
        };
        if frame_id == "bg" {
            return ArtifactKind::Background;
        }
        let master = self.book().spreads[s].pages[p]
            .master
            .as_ref()
            .and_then(|m| self.book().master_pages.iter().find(|x| &x.id == m));
        let frame = master.and_then(|m| m.frames.iter().find(|f| f.base().id.0 == frame_id));
        match frame {
            Some(Frame::Text(text)) => match printed_of(&text.style.0) {
                Some(printed) => self.furniture_kind(printed, &text.base.rect),
                None if text_is_a_number(frame.expect("matched")) => ArtifactKind::PageNumber,
                None => ArtifactKind::Layout,
            },
            _ => ArtifactKind::Layout,
        }
    }

    fn page(&mut self, sheet: usize, member: usize, page: &PageScene, out: &mut Emit) {
        let Some(&(spread_at, page_at)) = self.b().pages.get(page.page.0.as_str()) else {
            // A page the book does not know (a made-up scene): everything on it is decoration.
            for (item, _) in page.items.iter().enumerate() {
                self.mark_item(
                    ItemAt {
                        sheet,
                        member,
                        item,
                    },
                    page,
                    Class::Artifact(ArtifactKind::Layout),
                );
            }
            return;
        };
        let spread = &self.book().spreads[spread_at];
        let frames: HashMap<&str, &Frame> = folio_model::leaves(&spread.frames)
            .into_iter()
            .map(|f| (f.base().id.0.as_str(), f))
            .collect();
        let rects = page_rects(self.book(), spread);
        let left = rects.get(page_at).map_or(0.0, |r| r.rect.x.0);

        // Group the items of the page by frame.
        let mut entries: Vec<Entry> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        for (item_at, item) in page.items.iter().enumerate() {
            let base = base_of(&item.frame.0).to_owned();
            let at = *index.entry(base.clone()).or_insert_with(|| {
                let frame = frames.get(base.as_str()).copied();
                let bounds = frame.map_or((0.0, 0.0, 0.0, 0.0), |f| {
                    let b = f.base().rect.turned_bounds(f.base().rotation);
                    (b.x.0 - left, b.y.0, b.w.0, b.h.0)
                });
                let class = self.classify(spread_at, page_at, &base, frame);
                entries.push(Entry {
                    base: base.clone(),
                    art: None,
                    texts: Vec::new(),
                    frame,
                    bounds,
                    z: item_at,
                    class,
                });
                entries.len() - 1
            });
            match item.kind {
                SceneKind::Text { .. } => entries[at].texts.push(item_at),
                _ => entries[at].art = Some(item_at),
            }
        }

        // Decoration is marked where it is; content is read in order.
        let mut content: Vec<usize> = Vec::new();
        for (at, entry) in entries.iter().enumerate() {
            match entry.class {
                Class::Artifact(_) => {
                    let class = entry.class;
                    let items: Vec<usize> = entry.art.iter().chain(&entry.texts).copied().collect();
                    for item in items {
                        self.mark_item(
                            ItemAt {
                                sheet,
                                member,
                                item,
                            },
                            page,
                            class,
                        );
                    }
                }
                Class::Content => content.push(at),
            }
        }
        let order = reading_order(&entries, &content);
        let mut done: BTreeSet<usize> = BTreeSet::new();
        for at in order {
            if done.contains(&at) {
                continue;
            }
            // A story is read through all its frames on the page where its first one falls.
            let entry = &entries[at];
            let thread: Vec<usize> = match flow_of(entry.frame) {
                Some((flow, _)) => {
                    let mut chain: Vec<(u32, usize)> = content
                        .iter()
                        .filter_map(|&o| {
                            flow_of(entries[o].frame)
                                .filter(|(f, _)| f == &flow)
                                .map(|(_, slot)| (slot, o))
                        })
                        .collect();
                    chain.sort();
                    chain.into_iter().map(|(_, o)| o).collect()
                }
                None => vec![at],
            };
            for o in thread {
                if done.insert(o) {
                    self.content(sheet, member, page, &entries[o], out);
                }
            }
        }
    }

    /// Marks every item of a frame as `class`.
    fn mark_item(&mut self, at: ItemAt, page: &PageScene, class: Class) {
        let Class::Artifact(kind) = class else {
            return;
        };
        let item = &page.items[at.item];
        let plan = match &item.kind {
            SceneKind::Text { lines, .. } => {
                ItemPlan::Text(lines.iter().map(|_| LinePlan::Artifact(kind)).collect())
            }
            SceneKind::Background { .. } => ItemPlan::Artifact(ArtifactKind::Background),
            _ => ItemPlan::Artifact(kind),
        };
        self.plan.items.insert(at, plan);
    }

    /// Plans the art and the words of a content frame, and puts them in the tree.
    fn content(
        &mut self,
        sheet: usize,
        member: usize,
        page: &PageScene,
        entry: &Entry,
        out: &mut Emit,
    ) {
        let at = |item| ItemAt {
            sheet,
            member,
            item,
        };
        // The frame's own art.
        if let Some(item) = entry.art {
            match (&page.items[item].kind, entry.frame) {
                (SceneKind::Picture { asset, .. }, Some(frame)) => {
                    if asset.is_none() {
                        // A frame still waiting for its picture is a grey box.
                        self.plan
                            .items
                            .insert(at(item), ItemPlan::Artifact(ArtifactKind::Layout));
                    } else {
                        let alt = described(frame);
                        let leaf = self.leaf(at(item), None, 0..0);
                        self.plan.items.insert(at(item), ItemPlan::Figure(leaf));
                        out.figure(alt, leaf);
                    }
                }
                (SceneKind::Shape { .. }, Some(frame)) => match described(frame) {
                    Some(alt) => {
                        let leaf = self.leaf(at(item), None, 0..0);
                        self.plan.items.insert(at(item), ItemPlan::Figure(leaf));
                        out.figure(Some(alt), leaf);
                    }
                    None => {
                        self.plan
                            .items
                            .insert(at(item), ItemPlan::Artifact(ArtifactKind::Layout));
                    }
                },
                _ => {
                    // A table's tints and rules, or art with no frame to describe it.
                    self.plan
                        .items
                        .insert(at(item), ItemPlan::Artifact(ArtifactKind::Layout));
                }
            }
        }
        if entry.texts.is_empty() {
            return;
        }
        if let Some(Frame::Table(table)) = entry.frame {
            self.table(sheet, member, page, entry, table.head as usize, out);
            return;
        }
        let style = text_style(self.book(), entry.frame);
        let owner = entry.base.clone();
        for &item in &entry.texts {
            let SceneKind::Text { lines, .. } = &page.items[item].kind else {
                continue;
            };
            let mut plans: Vec<LinePlan> = Vec::new();
            for (line_at, line) in lines.iter().enumerate() {
                let pieces = self.pieces(at(item), line_at, line);
                plans.push(LinePlan::Pieces(pieces));
            }
            // The paragraphs: runs of lines from the same paragraph.
            let mut start = 0;
            while start < lines.len() {
                let paragraph = lines[start].paragraph;
                let mut end = start + 1;
                while end < lines.len() && lines[end].paragraph == paragraph && paragraph.is_some()
                {
                    end += 1;
                }
                let own_style = self.paragraph_style(entry.frame, paragraph, &style);
                let role = role_of(&own_style);
                // All the words of a title frame are one heading.
                let key_paragraph = if role == Role::Title {
                    u32::MAX
                } else {
                    paragraph.unwrap_or(u32::MAX - 1 - start as u32)
                };
                let key = (story_of(entry.frame, &owner), key_paragraph);
                let mut parts: Vec<(Vec<Piece>, String)> = Vec::new();
                for line_at in start..end {
                    if let LinePlan::Pieces(pieces) = &plans[line_at] {
                        parts.push((pieces.clone(), line_text(&lines[line_at].runs)));
                    }
                }
                out.paragraph(role, key, parts);
                start = end;
            }
            self.plan.items.insert(at(item), ItemPlan::Text(plans));
        }
    }

    /// The marks of a line: its runs cut where the language or the link changes.
    fn pieces(&mut self, at: ItemAt, line_at: usize, line: &LayoutLine) -> Vec<Piece> {
        let mut pieces = Vec::new();
        let mut start = 0;
        let runs = &line.runs;
        while start < runs.len() {
            let mut end = start + 1;
            while end < runs.len() && same_piece(&runs[start], &runs[end]) {
                end += 1;
            }
            let leaf = self.leaf(at, Some(line_at), start..end);
            pieces.push(Piece {
                runs: start..end,
                leaf,
                lang: runs[start].lang.clone(),
                href: runs[start].href.clone(),
            });
            start = end;
        }
        pieces
    }

    /// A table: header rows, then body rows, each cell holding its paragraphs.
    fn table(
        &mut self,
        sheet: usize,
        member: usize,
        page: &PageScene,
        entry: &Entry,
        head: usize,
        out: &mut Emit,
    ) {
        let at = |item| ItemAt {
            sheet,
            member,
            item,
        };
        let mut cells: Vec<(usize, usize, usize)> = Vec::new();
        for &item in &entry.texts {
            let id = &page.items[item].frame.0;
            if let Some((_, cell)) = id.split_once('#') {
                if let Some((r, c)) = cell.split_once('.') {
                    if let (Ok(r), Ok(c)) = (r.parse::<usize>(), c.parse::<usize>()) {
                        cells.push((r, c, item));
                    }
                }
            }
        }
        cells.sort();
        let rows: BTreeSet<usize> = cells.iter().map(|(r, _, _)| *r).collect();
        let mut table_rows: Vec<(bool, Vec<Node>)> = Vec::new();
        for r in rows {
            let header = r < head;
            let mut row_cells: Vec<Node> = Vec::new();
            for &(_, _, item) in cells.iter().filter(|(row, _, _)| *row == r) {
                let SceneKind::Text { lines, .. } = &page.items[item].kind else {
                    continue;
                };
                let mut plans = Vec::new();
                let mut cell = Emit::default();
                let mut start = 0;
                let mut line_plans: Vec<Vec<Piece>> = Vec::new();
                for (line_at, line) in lines.iter().enumerate() {
                    let pieces = self.pieces(at(item), line_at, line);
                    plans.push(LinePlan::Pieces(pieces.clone()));
                    line_plans.push(pieces);
                }
                while start < lines.len() {
                    let paragraph = lines[start].paragraph;
                    let mut end = start + 1;
                    while end < lines.len()
                        && lines[end].paragraph == paragraph
                        && paragraph.is_some()
                    {
                        end += 1;
                    }
                    let parts = (start..end)
                        .map(|l| (line_plans[l].clone(), line_text(&lines[l].runs)))
                        .collect();
                    // Each paragraph of a cell is its own element, whatever the key.
                    cell.paragraph(
                        Role::Paragraph,
                        (format!("{}:{start}", page.items[item].frame.0), 0),
                        parts,
                    );
                    start = end;
                }
                self.plan.items.insert(at(item), ItemPlan::Text(plans));
                row_cells.push(Node::Group(Group {
                    kind: if header { Kind::HeaderCell } else { Kind::Cell },
                    children: cell.finish(),
                }));
            }
            table_rows.push((header, row_cells));
        }
        let mut head_rows = Vec::new();
        let mut body_rows = Vec::new();
        for (header, cells) in table_rows {
            let row = Node::Group(Group {
                kind: Kind::Row,
                children: cells,
            });
            if header {
                head_rows.push(row);
            } else {
                body_rows.push(row);
            }
        }
        let mut children = Vec::new();
        if !head_rows.is_empty() {
            children.push(Node::Group(Group {
                kind: Kind::TableHead,
                children: head_rows,
            }));
        }
        if !body_rows.is_empty() {
            children.push(Node::Group(Group {
                kind: Kind::TableBody,
                children: body_rows,
            }));
        }
        out.close();
        out.nodes.push(Node::Group(Group {
            kind: Kind::Table,
            children,
        }));
    }

    fn paragraph_style(
        &self,
        frame: Option<&Frame>,
        paragraph: Option<u32>,
        frame_style: &str,
    ) -> String {
        if let (Some((flow, _)), Some(p)) = (flow_of(frame), paragraph) {
            if let Some(style) = self
                .book()
                .flows
                .iter()
                .find(|f| f.id == flow)
                .and_then(|f| f.paragraphs.get(p as usize))
                .and_then(|p| p.style.as_ref())
            {
                return style.0.clone();
            }
        }
        frame_style.to_owned()
    }
}

/// Frames in a story, by the story's id and the frame's place in it.
fn flow_of(frame: Option<&Frame>) -> Option<(folio_model::FlowId, u32)> {
    match frame {
        Some(Frame::Text(text)) => text.flow.as_ref().map(|slot| (slot.id.clone(), slot.index)),
        _ => None,
    }
}

/// What lines belong to the same paragraph across frames: a story's id, or else the frame's.
fn story_of(frame: Option<&Frame>, owner: &str) -> String {
    match flow_of(frame) {
        Some((flow, _)) => format!("flow:{}", flow.0),
        None => format!("frame:{owner}"),
    }
}

/// The paragraph style of a frame's own words.
fn text_style(book: &Book, frame: Option<&Frame>) -> String {
    let _ = book;
    match frame {
        Some(Frame::Text(text)) => text.style.0.clone(),
        Some(Frame::Shape(shape)) => shape
            .style
            .as_ref()
            .map_or_else(|| "bubble".into(), |s| s.0.clone()),
        Some(Frame::Table(table)) => table
            .style
            .as_ref()
            .map_or_else(|| "cell".into(), |s| s.0.clone()),
        _ => "body".to_owned(),
    }
}

fn described(frame: &Frame) -> Option<String> {
    frame
        .base()
        .description
        .as_ref()
        .map(|d| d.trim())
        .filter(|d| !d.is_empty())
        .map(str::to_owned)
}

/// The frame an item of the scene belongs to: a table's cells and a master's copies name their
/// frame before a `#`.
fn base_of(id: &str) -> &str {
    match id.split_once('#') {
        Some((table, _)) => table,
        None => id,
    }
}

fn text_is_a_number(frame: &Frame) -> bool {
    match frame {
        Frame::Text(text) => text.content.as_ref().is_some_and(|c| {
            is_page_number_text(&c.inline.iter().map(|r| r.text.as_str()).collect::<String>())
        }),
        _ => false,
    }
}

/// Whether two neighbouring runs of a line are marked together.
fn same_piece(a: &LayoutRun, b: &LayoutRun) -> bool {
    a.lang == b.lang && a.href == b.href
}

/// The words of a line as text: runs set apart by a gap are words, and a hyphen the line ends on
/// is not part of the text.
fn line_text(runs: &[LayoutRun]) -> String {
    let mut out = String::new();
    let mut end: Option<f64> = None;
    for run in runs {
        if run.hyphen || run.text.is_empty() {
            continue;
        }
        if let Some(end) = end {
            if run.x - end > 0.01 {
                out.push(' ');
            }
        }
        out.push_str(&run.text);
        end = Some(run.x + run.width);
    }
    out
}

/// The order a page's content frames are read in.
fn reading_order(entries: &[Entry], content: &[usize]) -> Vec<usize> {
    let mut sorted: Vec<usize> = content.to_vec();
    sorted.sort_by(|&a, &b| {
        let (ea, eb) = (&entries[a], &entries[b]);
        ea.bounds
            .1
            .total_cmp(&eb.bounds.1)
            .then(ea.bounds.0.total_cmp(&eb.bounds.0))
            .then(ea.z.cmp(&eb.z))
    });
    // Rows: a frame joins the row of the one that began it when they overlap in height by more than
    // half the shorter of the two.
    let mut rows: Vec<Vec<usize>> = Vec::new();
    for at in sorted {
        let (_, top, _, height) = entries[at].bounds;
        let joins = rows.last().is_some_and(|row| {
            let (_, anchor_top, _, anchor_height) = entries[row[0]].bounds;
            let overlap = (top + height).min(anchor_top + anchor_height) - top.max(anchor_top);
            let shorter = height.min(anchor_height);
            overlap > shorter / 2.0
        });
        if joins {
            rows.last_mut().expect("a row").push(at);
        } else {
            rows.push(vec![at]);
        }
    }
    // Inside a row: left to right, frames at about the same left edge from the top down.
    const SAME_COLUMN: f64 = 8.0;
    for row in &mut rows {
        row.sort_by(|&a, &b| {
            let (ea, eb) = (&entries[a], &entries[b]);
            let column = |e: &Entry| (e.bounds.0 / SAME_COLUMN).round() as i64;
            column(ea)
                .cmp(&column(eb))
                .then(ea.bounds.1.total_cmp(&eb.bounds.1))
                .then(ea.z.cmp(&eb.z))
        });
    }
    rows.into_iter().flatten().collect()
}

/// Builds the top level of the tree, folding the lines of one paragraph into one element.
#[derive(Default)]
struct Emit {
    nodes: Vec<Node>,
    /// The paragraph that may take more lines: its key, and that it is the last node.
    open: Option<(String, u32)>,
}

impl Emit {
    fn close(&mut self) {
        self.open = None;
    }

    fn figure(&mut self, alt: Option<String>, leaf: LeafId) {
        self.close();
        self.nodes.push(Node::Group(Group {
            kind: Kind::Figure { alt },
            children: vec![Node::Leaf(leaf)],
        }));
    }

    /// Adds lines of the paragraph `key` (a story or frame, and a paragraph of it), starting a new
    /// element unless the last one is the same paragraph carrying on.
    fn paragraph(&mut self, role: Role, key: (String, u32), parts: Vec<(Vec<Piece>, String)>) {
        let carrying = self.open.as_ref() == Some(&key);
        // A blank line is nothing to read.
        if parts.iter().all(|(pieces, _)| pieces.is_empty()) {
            return;
        }
        if !carrying {
            let after_figure = matches!(
                self.nodes.last(),
                Some(Node::Group(Group {
                    kind: Kind::Figure { .. },
                    ..
                }))
            );
            let kind = match role {
                Role::Title => Kind::Heading {
                    level: 1,
                    title: String::new(),
                },
                Role::Subheading => Kind::Heading {
                    level: 2,
                    title: String::new(),
                },
                Role::Epigraph => Kind::BlockQuote,
                Role::Note => Kind::Note,
                Role::Caption if after_figure => Kind::Caption,
                _ => Kind::Paragraph,
            };
            debug_assert_eq!(
                heading_level(role).is_some(),
                matches!(kind, Kind::Heading { .. })
            );
            self.nodes.push(Node::Group(Group {
                kind,
                children: Vec::new(),
            }));
            self.open = Some(key);
        }
        let Some(Node::Group(group)) = self.nodes.last_mut() else {
            return;
        };
        for (pieces, text) in parts {
            if let Kind::Heading { title, .. } = &mut group.kind {
                if !title.is_empty() && !text.is_empty() {
                    title.push(' ');
                }
                title.push_str(&text);
            }
            for piece in pieces {
                match &piece.href {
                    // Words that link are one Link element across the lines they run over.
                    Some(href) => match group.children.last_mut() {
                        Some(Node::Group(Group {
                            kind: Kind::Link { href: last, .. },
                            children,
                        })) if last == href => {
                            children.push(Node::Leaf(piece.leaf));
                        }
                        _ => group.children.push(Node::Group(Group {
                            kind: Kind::Link {
                                href: href.clone(),
                                id: piece.leaf,
                            },
                            children: vec![Node::Leaf(piece.leaf)],
                        })),
                    },
                    None => group.children.push(Node::Leaf(piece.leaf)),
                }
            }
        }
    }

    fn finish(self) -> Vec<Node> {
        self.nodes
    }
}

#[cfg(test)]
mod tests;
