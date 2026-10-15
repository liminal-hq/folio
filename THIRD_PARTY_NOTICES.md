# Third-party notices

Folio is `Apache-2.0 OR MIT`. These parts of what it ships carry notices of their own. The licence check (`deny.toml`, `scripts/check-licences.sh`) keeps any other licence out of the dependencies.

## Mozilla Public License 2.0 crates

MPL-2.0 is a file-level licence. Each of these is used unmodified, as a dependency, and its source is available from the address shown. The licence text is at <https://www.mozilla.org/MPL/2.0/>.

- `spellbook` 0.4.2, the Hunspell-compatible spell checker used by `folio-spell`: <https://github.com/helix-editor/spellbook>.
- `cssparser`, `cssparser-macros`, `selectors`, `dtoa-short` and `option-ext`, brought in by Tauri's web stack and by its dependencies.

## English (Canada) dictionary

`crates/folio-spell/dictionaries/en-CA/` holds the `en_CA` Hunspell dictionary from SCOWL (Spell Checker Oriented Word Lists) in the form published as `dictionary-en-ca` 3.0.0 by `wooorm/dictionaries` (licence `(MIT AND BSD)`). SCOWL requires its copyright notice to stay with the dictionary and with anything derived from it, so the whole notice is in `crates/folio-spell/dictionaries/en-CA/LICENSE`, and its provenance is in `PROVENANCE.md` beside it.
