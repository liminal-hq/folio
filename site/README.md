# Folio documentation site

The public site for Folio, built with [Astro Starlight](https://starlight.astro.build) and deployed to GitHub Pages at `https://liminal-hq.github.io/folio/`.

- `src/pages/index.astro` is the hand-built landing page.
- `src/content/docs/manual/` is the manual, as Markdown in three groups (getting started, walkthrough, using Folio). The sidebar is generated from the folders and each page's `sidebar.order`.
- `src/styles/folio.css` maps Folio's colours onto Starlight's.

```bash
bun run site:dev     # live preview
bun run site:build   # static build into site/dist
```

The site is served from `/folio` by default. Set `PAGES_BASE_PATH` (an empty value serves from the root) and `SITE_URL` for a preview or a custom domain. The manual describes how Folio is designed to work; pages change as the app is built.
