# Third-party notices

Folio is `Apache-2.0 OR MIT`. These parts of what it ships carry notices of their own. The licence check (`deny.toml`, `scripts/check-licences.sh`) keeps any other licence out of the dependencies.

## Mozilla Public License 2.0 crates

MPL-2.0 is a file-level licence. Each of these is used unmodified, as a dependency, and its source is available from the address shown. The licence text is at <https://www.mozilla.org/MPL/2.0/>.

- `spellbook` 0.4.2, the Hunspell-compatible spell checker used by `folio-spell`: <https://github.com/helix-editor/spellbook>.
- `cssparser`, `cssparser-macros`, `selectors`, `dtoa-short` and `option-ext`, brought in by Tauri's web stack and by its dependencies.

## Root certificates

`webpki-roots` carries Mozilla's list of trusted certificate authorities, under CDLA-Permissive-2.0 (<https://cdla.dev/permissive-2-0/>). Folio's downloads of dictionaries use it, through `ureq`, to know which servers to trust.

## Downloaded dictionaries

Other dictionaries are downloaded when a person asks for them, never bundled, and kept as separate files with their own licence file beside them. Each is pinned by version and hash in `crates/folio-spell/data/dictionaries.json`, which also records its licence.

## English (Canada) dictionary

`crates/folio-spell/dictionaries/en-CA/` holds the `en_CA` Hunspell dictionary from SCOWL (Spell Checker Oriented Word Lists) in the form published as `dictionary-en-ca` 3.0.0 by `wooorm/dictionaries` (licence `(MIT AND BSD)`). SCOWL requires its copyright notice to stay with the dictionary and with anything derived from it, so the whole notice is in `crates/folio-spell/dictionaries/en-CA/LICENSE`, and its provenance is in `PROVENANCE.md` beside it.

## Bundled fonts

Folio ships four typeface families so a book looks the same on every computer: Literata, Source Serif 4, Inter and Atkinson Hyperlegible. Each is under the SIL Open Font License 1.1, and its full licence text is kept beside the font files in `crates/folio-layout/fonts/` (`OFL-literata.txt`, `OFL-sourceserif4.txt`, `OFL-inter.txt`, `OFL-atkinsonhyperlegible.txt`). The licence lets the fonts be embedded in the PDFs and EPUBs a book is exported as, and kept inside a `.folio` file, as Folio does, as long as they are not sold on their own. Folio does not rename or modify them.

## Hyphenation patterns

`hyphenation` (Apache-2.0 or MIT) brings in the TeX `hyph-utf8` patterns, which are the work of their respective owners under their own licences (mostly MIT, with some LPPL and others); the licence of each language's pattern file is in the `hyph-utf8` sources it was built from, <https://github.com/hyphenation/tex-hyphen>. Folio uses the patterns for the languages it checks spelling in.

## Colour and PDF libraries

Little CMS (`lcms2`, MIT, <https://www.littlecms.com>) is built into Folio to convert colours and soft-proof; `krilla` and `pdf-writer` (Apache-2.0 or MIT) write PDFs; `tiny-skia` (BSD-3-Clause) draws pictures of pages; `lopdf` (MIT) completes PDF/X files. No ICC colour profile for a printing press ships with Folio. The sRGB profile used in PDF/X files is Little CMS's own built-in one.
