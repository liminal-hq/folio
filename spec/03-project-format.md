# 03 · The `.folio` project format

A `.folio` file is a **zip package modelled on ODF/EPUB OCF**. It is easy to write, easy to read, and can be unpacked by anyone with no special tools.

## Layout inside the zip

```
mimetype                         # first entry, STORED (no compression), no extra field
                                 # contents: application/vnd.liminal.folio+zip
META-INF/manifest.json           # every file, its media type, sha256, role
book.json                        # the document model (see types/), pretty-printed, stable key order
styles.json                      # paragraph + character styles, colour swatches
masters.json                     # master pages (frames, `guides` and `bg`; see D-22, D-23)
text/
  flow-<id>.md                   # long text flows as CommonMark + Folio attributes (see below)
assets/
  images/<sha256>.<ext>          # original files, deduped by content hash
  fonts/<family>/<file>.ttf      # only when the user chose "Keep fonts with the book"
  icc/<name>.icc                 # output intents used by the book
thumbnails/
  cover.png                      # 512 px, for file managers (Waypoint, Nautilus, Dolphin, Explorer)
  spread-<n>.png                 # small, optional, regenerated on save
recovery/                        # reserved: ignored on open, never written (see D-27)
settings.json                    # per-book export settings, last target, preflight ignores
```

### Rules

- The `mimetype` entry must be first and stored, as in ODF/EPUB. This lets `file(1)` and file managers sniff it.
- JSON is UTF-8 with no BOM, 2-space indentation, and keys in schema order. That keeps diffs readable, so a `.folio` unpacked into Git is pleasant.
- Every path in `book.json` is relative to the package root.
- Unknown files and unknown JSON keys are **preserved** on save (forward compatibility).
- The writer streams to `<name>.folio.tmp` and fsyncs, then renames atomically. A `<name>.folio~` backup of the previous save is kept (Preferences can turn it off).

## `META-INF/manifest.json`

```json
{
  "format": "folio",
  "formatVersion": "1.0",
  "generator": "Folio 0.1.0 (Liminal HQ)",
  "created": "2026-10-13T14:21:00Z",
  "modified": "2026-10-13T16:02:11Z",
  "files": [
    { "path": "book.json", "mediaType": "application/json", "role": "book" },
    { "path": "assets/images/9f2c…e1.jpg", "mediaType": "image/jpeg", "sha256": "9f2c…e1", "role": "image" }
  ]
}
```

## `book.json` (abridged)

```json
{
  "schema": 1,
  "id": "bk_01J9…",
  "title": "The Otter Who Couldn't Sleep",
  "authors": [{ "name": "Mom", "role": "author" }],
  "language": "en-CA",
  "kind": "picture",
  "layout": "fixed",
  "trim": { "w": 612, "h": 612 },
  "bleed": 9,
  "margins": { "top": 36, "bottom": 36, "inside": 36, "outside": 36 },
  "grid": { "columns": 6, "gutter": 12, "baseline": null },
  "spreads": [
    { "id": "sp_cover", "kind": "cover", "pages": [{ "id": "pg_c", "master": null }],
      "frames": [
        { "id": "fr_c1", "type": "image", "box": { "x": -9, "y": -9, "w": 630, "h": 630 },
          "image": { "asset": "assets/images/9f2c…e1.jpg", "fit": "fill", "focus": [0.5, 0.4] } },
        { "id": "fr_t1", "type": "text", "box": { "x": 54, "y": 61.2, "w": 504, "h": 144 },
          "style": "display", "content": { "inline": "The Otter Who Couldn't Sleep" } }
      ] }
  ],
  "flows": [{ "id": "fl_main", "source": "text/flow-main.md", "frames": ["fr_b1", "fr_b2", "fr_b3"] }],
  "metadata": { "isbn": { "paperback": null, "ebook": null }, "series": null, "description": "" }
}
```

A page may carry a `bg` (a background, spec 11), a spread a `bgSpan` across both pages, and a master a `bg` for the pages that use it. All three are omitted when there is none, so a book without backgrounds is unchanged.

All lengths are in **points** (1/72 in), and coordinates are spread-relative with the origin at the top-left of the trim box. Negative values reach into the bleed.

## Text flows (`text/flow-*.md`)

Long text is stored as CommonMark so that it stays readable outside Folio, plus a small attribute syntax:

```markdown
{style=label}
Chapter one

# The keeper's kettle {style=h1 .chapter-start}

Mara had kept the light at Gannet Point for eleven winters…

::: {style=callout}
Try this · Wet a postcard-sized sheet edge to edge…
:::
```

- `{style=…}` attaches a paragraph style; `{.class}` attaches semantic roles (`chapter-start`, `no-indent`, `keep-with-next`).
- Character styles use `[text]{style=emph}`. Plain `*em*` and `**strong**` map to the defaults.
- Short text (picture book captions, cover titles) stays inline in `book.json` (`content.inline`).

## Versioning (D-05)

- `schema` is an integer. Migrations live in `folio-model::migrate` and run on open. Files are never rewritten silently: the first save after a migration asks "Save in the new format? Older Folio versions won't open it."
- `formatVersion` is the container version (zip rules). It changes rarely.

## Interop promises

- Unzipping a `.folio` gives you your words (`text/*.md`, or inline in `book.json`) and your original pictures (`assets/images/`). This is the sovereignty guarantee.
- `folio export --format folder` writes the same tree unzipped, which is handy for Git.
- A thumbnailer for Linux (`folio-thumbnailer`, a `.thumbnailer` file + small binary) reads `thumbnails/cover.png`. Windows gets an `IThumbnailProvider` shell extension later.
