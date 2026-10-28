# Folio

<p align="center">
  <img src="assets/hero.svg" alt="Folio — books, laid out by hand, without the hard parts" width="100%">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-early%20development-f97316" alt="Status: early development">
  <img src="https://img.shields.io/badge/platforms-Linux%20%C2%B7%20Windows%2011-1f6feb" alt="Platforms: Linux and Windows 11">
  <img src="https://img.shields.io/badge/licence-Apache--2.0%20OR%20MIT-3fb950" alt="Licence: Apache-2.0 OR MIT">
</p>

Folio is a local-first book layout app for Linux and Windows 11. It sits somewhere between Scribus and a friendly word processor: in **Easy** mode, someone who has never heard of bleed can make a picture book and send it to Amazon KDP, and **Expert** mode exposes baseline grids, master pages, PDF/X and colour profiles.

> **Status:** early development. The product specification and a design prototype are written, and the application is being scaffolded from the window outwards. There is nothing to download yet.

## What Folio is for

- **Books of every kind.** Picture books, novels, textbooks, cookbooks and illustrated guides, poetry and memoirs with photos, each starting from a preset with the right trim size, bleed, grid and styles.
- **Easy or Expert.** Easy talks in plain language ("pink dashed line") and hides what you don't need. Expert shows everything ("bleed 0.125 in"). Switching never removes data.
- **From Word to Kindle, Kobo and print.** Bring in words from Word, Google Docs, ODT, Markdown or a paste, then export EPUB for Kindle and Kobo, KDP-ready interior and cover PDFs, PDF/X for print shops, and a share PDF — with fix-oriented preflight before every export.
- **Covers that do the maths.** Spine width from page count and paper, barcode-safe areas, bleed and wrap for paperback, hardcover and dust jacket.
- **Yours to keep.** A `.folio` file is an open zip of JSON, Markdown and your original pictures. Nothing leaves your computer unless you export it, and there is no account.
- **Recognisable on your desktop.** Installed Linux packages and Windows installers give `.folio` books their own document icon in file explorers.
- **Jax, if you want him.** An optional 3D otter companion gives gentle tips, a per-page checklist and offline "how do I…?" answers from a local model.

## Platforms

| Platform                                             | Status                                                                    |
| ---------------------------------------------------- | ------------------------------------------------------------------------- |
| Linux (GNOME, KDE Plasma, Cinnamon; Wayland and X11) | Primary target. Wayland first.                                            |
| Windows 11                                           | A real target, written alongside the Linux code rather than ported later. |
| macOS                                                | Not planned for v1.                                                       |

## Trying it from source

Folio is not ready for daily use, but you can run the current shell.

You will need [Bun](https://bun.sh), a Rust toolchain (the version is pinned in `rust-toolchain.toml`), and the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your system. On Linux, that includes WebKitGTK 4.1.

```bash
bun install
bun run tauri:dev
```

`tauri:dev` starts the desktop shell with the development settings the MCP automation tooling needs. To run the whole local check suite before contributing:

```bash
bun run validate
```

If Rust tooling is not installed locally, run the Rust and Tauri commands in the `ghcr.io/liminal-hq/tauri-dev-desktop:latest` container.

## Documentation

| Location                                 | What it covers                                                                    |
| ---------------------------------------- | --------------------------------------------------------------------------------- |
| [`spec/`](spec/README.md)                | The specification: product, architecture, project format, layout, export, UI, Jax |
| [`docs/decisions.md`](docs/decisions.md) | The decision log (D-numbers) lifted from the specification                        |
| [`design/`](design/)                     | The Claude Design prototype files — reference only, not the source of truth       |
| [`site/`](site/)                         | The public documentation site                                                     |

## Contributing

Folio follows the Liminal HQ house conventions: Canadian English, Conventional Commits, human-readable pull request titles, licence headers on source files, and no pushes to `main`. The full rules are in [`AGENTS.md`](AGENTS.md) (mirrored for Claude Code in [`CLAUDE.md`](CLAUDE.md)).

## Part of Liminal HQ

Folio sits alongside [Waypoint](https://github.com/liminal-hq/waypoint), [Spindle](https://github.com/liminal-hq/spindle), [Jar](https://github.com/liminal-hq/jar) and [Cadence](https://github.com/liminal-hq/cadence), and shares their window chrome, tokens and conventions.

## Licence

Licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT licence](LICENSE-MIT), at your option.
