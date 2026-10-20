# AGENTS.md

## Table of Contents

- [Project Status](#project-status)
- [Localization and Spelling](#localization-and-spelling)
- [Markdown Formatting](#markdown-formatting)
- [Authoring Voice](#authoring-voice)
- [Commit Messages](#commit-messages)
- [Pull Request Titles](#pull-request-titles)
- [Pull Request Content](#pull-request-content)
- [Pull Request Labels](#pull-request-labels)
- [Git Workflow](#git-workflow)
- [Local Tooling](#local-tooling)
- [CI and Release](#ci-and-release)
- [Frontend Code Conventions](#frontend-code-conventions)
- [Architecture Rules](#architecture-rules)
- [Documentation](#documentation)
- [Repository Layout](#repository-layout)
- [Licence and Copyright](#licence-and-copyright)
- [Tauri v2](#tauri-v2)

## Project Status

Folio is a local-first book layout app for Linux (primary) and Windows 11 (a real target), built with Tauri v2, React/TypeScript and Rust. The repository is in **early development**: the product and architecture are specified in `spec/`, and the application is being scaffolded from the shell outwards (a frameless window with the shared Liminal chrome, settings, child windows), followed by the document model, the spread canvas, the ribbon and the exporters. Scaffold only what the current milestone needs, not the whole tree speculatively.

The prototype that the product design came from lives in a Claude Design project, and a copy of its files is kept in `design/`. It is **reference only**: a collection of stories about the app, not the source of truth. Only the _behaviour_ it demonstrates is authoritative, and only via `spec/`. Do not port, adapt or structurally mirror its code.

## Localization and Spelling

**REQUIREMENT:** All UI strings, code variables, comments, commit messages, pull request descriptions, and documentation MUST use **Canadian English** spelling.

Examples:

- `colour` instead of `color`
- `centre` instead of `center`
- `neighbour` instead of `neighbor`
- `behaviour` instead of `behavior`
- `cancelled` instead of `canceled`
- `customise` instead of `customize`, `minimise` instead of `minimize`
- `licence` (noun) vs `license` (verb) — though in UI context usually "Licence"

Keep American spellings where an external API, CSS property, crate or protocol requires them (for example `color`, `background-color`, `color-scheme`, `Cancel` in a third-party enum).

Capitalisation in the UI: sentence case for app actions and labels, Title Case for window titles, and no all-caps titles, headings or labels.

## Markdown Formatting

**REQUIREMENT:** Do not hard-wrap markdown prose. Write each paragraph or bullet as a single unwrapped line in the source, no matter how long — let the renderer (GitHub, a browser, an editor's soft-wrap) reflow it for display. This applies to markdown-rendered content: PR descriptions, docs under `docs/` and `spec/`, README files. Commit message bodies are the one exception — hard-wrap those (see [Commit Messages](#commit-messages)); `git log`/`git show` in a terminal don't reflow long lines the way GitHub's PR view does.

- Manual line breaks mid-paragraph don't survive Markdown rendering as intended, and they create noisy diffs when a later edit only changes one word but reflows the whole wrapped block.
- This does not apply to genuinely separate list items, headings, or intentional line breaks (e.g. two-space trailing breaks, blank lines between paragraphs) — only to breaking up one continuous sentence/paragraph across multiple lines.
- Code comments are not markdown-rendered and may wrap normally at a reasonable line length, same as any other source line — this rule doesn't reach them.

**Em dashes:** use a real em dash (`—`) in prose, never `--` as a substitute. Applies everywhere prose appears — PR/issue descriptions, docs, README files, commit messages, and code comments — independent of the hard-wrap rule's narrower scope above. Doesn't apply to an actual double-hyphen that means something else in context (a CLI flag like `--check`, a numeric range, etc.).

## Authoring Voice

**REQUIREMENT:** Ship the result, not how the conversation arrived at it. Write every outward-facing line — code comments, identifier names, docs, PR descriptions — as the author of the artifact, for the reader who will encounter it later, not as a record of the debugging or review process that produced it.

- Don't reference "this PR", "the review", a reviewer's name, or a commit SHA inside code comments or PR prose. State the fact or the reasoning directly, as if it had always been true.
- When a comment gets edited more than once across a change, rewrite it as one clean explanation — don't leave layered fragments from each edit stacked on top of each other.
- Commit messages are the exception: they're a legitimate place to record _why_ a change happened, including review feedback or debugging context — that's what git history is for.

## Commit Messages

**Format:** Use Conventional Commits format (e.g., `feat: ...`, `fix: ...`, `docs: ...`, `test: ...`).

- Use `test:` for test-related changes, including fixes to tests themselves (do not use `fix:` unless it fixes application code).
- Use `ci:` for workflow changes, `build:` for dependency and toolchain changes, `chore:` for housekeeping.

**Body Requirements:**

- Explain what and why (not how)
- Use markdown: **bold**, _italics_, `code`, bullet lists
- **Backtick every code-level reference** — component/function/class/variable names, file and directory paths, CSS selectors/properties/values, npm and crate package names, route paths, HTML tag names, config keys, and CLI flags (e.g. `WindowFrame`, `plugins/window-tearoff`, `:hover`, `overflow-x: hidden`, `@tanstack/react-router`, `<button>`, `--flag`). This applies inline in prose, not just in fenced code blocks. Plain-English descriptions and user-facing UI strings (button labels, screen names, dialog copy) use quotes instead, not backticks — they aren't code.
- **NO markdown headings** — use **bold labels** for sections (not always required)
- Hard-wrap paragraphs (~72-100 chars), unlike other markdown in this repo — see [Markdown Formatting](#markdown-formatting)

**Specific Updates:** Each commit message should reflect the specific changes made in that commit. Do not just recap the entire project history or scope. Focus on the now.

**Shell Interpolation Safety:**

- Do not pass markdown-heavy commit bodies directly via `git commit -m "..."` when they include backticks, `$()`, or shell-sensitive characters.
- Prefer writing the message to a file with a single-quoted heredoc and commit with `git commit -F <file>` to prevent shell expansion.
- After committing, verify the stored message with `git log -1 --pretty=fuller` and amend immediately if interpolation altered content.

## Pull Request Titles

**REQUIREMENT:** PR titles MUST be human-readable summaries of the PR change.

- Start with a capital letter, write in the imperative mood, and keep to roughly one 70-character line.
- Do not use Conventional Commit prefixes in PR titles (for example, no `feat:`, `fix:`, `chore:`).
- Describe the outcome or behaviour change, not internal process language.
- Keep title style consistent across every open PR in the same stack. If one title in a stack is updated, update the rest to match.
- Do not rename merged PRs unless explicitly requested.

## Pull Request Content

**Requirement:** PR titles and descriptions must not mention internal workflow artefacts.

- Do not mention deferred-review documents, internal queue labels, or internal-only planning notes in outward PR content.
- Use user-facing, outcome-focused language in PR titles and descriptions.
- Open pull requests ready for review by default. Only create a draft PR when the user explicitly asks for a draft or when there is a clearly communicated blocker that makes draft status necessary.

**PR Description Format:**

- Prefer a compact markdown structure with `## Summary` and `## Test plan`.
- Under `## Summary`, use `###` sub-sections when they help group the change cleanly (for example `### User-facing changes`, `### Plugins`, `### Documentation`).
- Under each summary section, use flat bullets with bold lead-ins for scanability.
- Keep the summary focused on outcomes and behaviour changes, not commit history or implementation chronology.
- Under `## Test plan`, use checklist bullets (`- [x]` / `- [ ]`) and include the concrete commands, validations, or remaining gaps.
- If something could not be verified, state that plainly at the end of `## Test plan` or immediately below it.

## Pull Request Labels

**Requirement:** Add labels to every PR when it is created or updated.

- Add at least one primary category label to every PR: `enhancement`, `bug`, `documentation`, `testing`, `ci`, `build`, or `chore`.
- Add shared operational labels where they help clarify handling: `infrastructure`, `internal`, `release`, `blocked`, `epic`, or `skip-changelog`.
- Add scope labels where helpful. Technology: `frontend`, `backend`, `rust`, `plugin`, `data-model`. Platform: `linux`, `windows`, `wayland`. Product area: `layout-engine`, `export`, `import`, `covers`, `typesetting`, `jax`, `theming`, `accessibility`, `localisation`, `design-system`, `performance`, `security`. Structure and process: `architecture`, `spike`, `experimental`, `developer-experience`.
- Prefer the broader Liminal HQ label style over Conventional Commit terms. Use `enhancement` and `bug`, not `feat` or `fix`.
- Use `skip-changelog` only when a change should be excluded from generated release notes (the categories are defined in `.github/release.yml`).
- Keep labels accurate as scope changes during review.

## Git Workflow

**Requirement:** Do not push changes (especially force pushes) to the repository unless explicitly requested by the user.

- **Default branch:** `main`. Work on a topic branch; do not commit directly to `main`.
- **Stacked work:** large changes are built as a stack of small branches with `gh stack` (`gh stack init`, `gh stack add`, `gh stack view`); `gh stack submit` pushes and opens the PRs, so it needs the same explicit go-ahead as a push.
- **Fix branch naming:** When creating a branch for a fix, use `fix/issue-<number>-<short-description>`.
- **GitHub tooling:** Prefer the `gh` CLI for repository, pull request, label, review, and GitHub Actions work.

## Local Tooling

- **JS runtime and package manager:** **Bun workspaces** (not pnpm/npm), with the Node version pinned in `.node-version` for tooling that needs it. Formatting is Prettier (`.prettierrc`: tabs, single quotes, 100 columns).
- **Rust:** the toolchain is pinned in `rust-toolchain.toml`. If `cargo` is not available on the host, run Rust/Tauri commands in the `ghcr.io/liminal-hq/tauri-dev-desktop:latest` container against the checked-out workspace.
- **Validation gate:** `bun run validate` is the single local gate that mirrors CI and must pass before opening or updating a PR. It runs the format check, the licence-header check, the case-collision and browser-storage checks, `tsc`, Vitest, the app build, `cargo fmt`, `cargo clippy -D warnings` and `cargo nextest`.
- **Editor settings:** `.editorconfig` is authoritative (tabs, LF, UTF-8).

### MCP Automation Bridge

An agent driving the running app (screenshots, DOM snapshots, JS eval, simulated input) uses the `tauri-mcp-server` MCP tool, which talks to `tauri-plugin-mcp-bridge` — a debug-only plugin registered under `#[cfg(all(debug_assertions, not(windows)))]` that never exists in release builds. Start the app with `bun run tauri:dev`, **not** plain `tauri dev`: `tauri:dev` merges `src-tauri/tauri.conf.dev.json`, which sets `"withGlobalTauri": true`, the global the bridge's JS-eval callback needs. Production configuration never sets it. Without it, `webview_execute_js`, screenshots, DOM snapshots and `webview_wait_for` silently time out while non-JS calls keep working. Folio is multi-window, so target windows by label. Do not set `WEBKIT_DISABLE_DMABUF_RENDERER=1` in the script: with WebKitGTK 2.54 it forces the old software path, and a frameless transparent window then repaints only the regions that changed and drops the rest of the page. Leave the DMABUF renderer on, and re-test if the Web Inspector renders black on Wayland.

### Logging

Native Rust and webview output share one log stream. `tauri-plugin-log` is configured in `src-tauri/src/lib.rs` (`Trace` in debug builds, `Info` in release; stdout plus a rotating `Folio.log`), and `src/services/logger.ts` redirects every webview `console.*` call into it, prefixed with the window label, and also logs uncaught errors and unhandled promise rejections, which never pass through `console`. Rules:

- Every window entry calls `initLogger(label)` once at startup; new window kinds must too.
- Use `log::{trace,debug,info,warn,error}!` in Rust and `console.*` in the front end. Do not add a second logging path, and never log secrets, credentials or book text.
- Each window's capabilities must include `log:default` (`capabilities/logging.json` grants it to every window), or forwarding fails silently.
- The log file is under the app's log directory (on Linux, `~/.local/share/ca.liminalhq.folio/logs/`).

## CI and Release

CI and release follow the Liminal HQ house pipeline (Waypoint, Jar and Cadence are the references).

- **Workflows** live in `.github/workflows/`: `ci.yml` (quality gate on every PR and push to `main`), and later a JUnit-publishing `workflow_run` workflow, `release.yml` (tag-driven cross-platform bundles) and the docs-site deploy.
- **Rust jobs run in the `ghcr.io/liminal-hq/tauri-ci-desktop:latest` container** with `Swatinem/rust-cache`; JS jobs use `oven-sh/setup-bun`. Installs use `--frozen-lockfile`.
- **Generated-code drift is a CI failure.** The `ts-rs` output is regenerated from scratch in CI and diffed against the committed copy.
- **Licence headers** are enforced by `scripts/check-headers.sh`.
- **Releases** are tagged `vX.Y.Z` (or `vX.Y.Z-beta.N`), must be on `main`, and every manifest version must be synchronised before bundles are built. Do not hand-bump a single manifest.
- **Dependabot** groups `tauri*` / `@tauri-apps/*` updates and labels them `build` (npm, cargo) or `ci` (Actions).
- Never push tags, trigger releases or dispatch workflows unless explicitly asked.

## Frontend Code Conventions

- **No barrel files.** Don't create an `index.ts`/`index.tsx` that only re-exports from sibling files. Import directly from the file that defines the thing. Barrels obscure the real dependency graph and slow down tree-shaking and IDE "go to definition."
- **React 19 + TypeScript (strict) + Vite.** Function components only. One folder per area directly under `src/` (`app`, `windows`, `settings`, `services`, …).
- **The frontend renders; Rust decides.** Document, layout, export and settings logic lives in Rust. Frontend code reacts to commands and events and never re-derives domain rules. Ephemeral UI state (hover, drag pointer state, focus) stays in the frontend.
- **No browser storage for app state.** Preferences and window state go through the settings plugin; `localStorage`, `sessionStorage` and IndexedDB are not used. `scripts/check-browser-storage.sh` enforces it.
- **Styling:** CSS Modules plus CSS custom properties. Folio's tokens use the `--folio-*` prefix (and `data-folio-theme`), never Waypoint's `--wp-*`. No inline-style styling: a value that must be measured in script, such as a menu's position, is passed to the stylesheet as a CSS custom property (`style={{ '--folio-menu-x': '12px' }}`) and consumed by a rule in the module. **Colour literals live only in token files** (`apps/folio/src/theme/tokens.css` and the chrome's `packages/chrome/src/tokens.css` defaults); component CSS uses `var(--folio-*)` with no inline fallback. Book pages are never themed: paper stays paper (`--folio-paper`, `--folio-ink`).
- **Cursors:** a desktop app shows the plain arrow over buttons, tabs, cards and menu items, and uses the pointing hand only for a real link that opens somewhere. Set `cursor: default` on controls (as the chrome package does), never `cursor: pointer`. Direct-manipulation surfaces keep their own cursors (move, resize, crosshair).
- **Accessibility is a build requirement, not a polish pass:** roles, names, focus order and keyboard paths follow `spec/06-ui.md`. A component that is not keyboard-operable is not done.
- **All user-visible strings go through the message catalogue** (`apps/folio/src/i18n/messages.ts`, `t('area.thing')`) — no string literals in JSX. The chrome's labels are supplied from it.
- **JavaScript reaches a native plugin only through its `guest-js` package.** Application and package code imports typed functions and never calls `invoke('plugin:...|...')` or listens for a plugin's event by string. `scripts/check-plugin-boundaries.sh` enforces it.
- **Module names never differ only by case.** `FrameMenu.tsx` beside `frameMenu.ts` is two files on Linux and one on Windows, where `tsc` fails. Give the pure model of a component a distinct name (`frameMenuModel.ts`). `scripts/check-case-collisions.sh` (`bun run check:case`) enforces it.

## Architecture Rules

- **Concern-first split.** Each concern (document model, package, layout, export, import, colour, Jax, settings, tear-off, window effects, system appearance) is its own crate or Tauri plugin. The app crate composes them; it does not implement concerns itself.
- **Pure crates for logic, thin plugins for the Tauri surface.** Logic that is large, reusable outside Tauri, or worth testing headlessly lives in a `crates/*` crate with **no `tauri` dependency**. A plugin is the adapter: commands, events, managed state, permissions.
- **Plugins never call each other.** Cross-concern behaviour is composed in the app crate through traits defined in the pure crates.
- **Generic plugins stay generic, and published ones are dependencies.** A reusable plugin must not import any Folio crate or mention Folio concepts. If a plugin is published to crates.io and npm (`system-appearance`, `os-prefs`, `window-manager`), Folio depends on the release. Only unpublished plugins are copied into `plugins/`, each with a `PROVENANCE.md`, and the copy is deleted once the plugin is published.
- **Rust owns state (D-02).** One writer per piece of state, a monotonic revision on every mutation, and granular events after each change. Streaming data (export progress, layout) uses Tauri `Channel`s, not polled commands.
- **Shared types are generated.** Wire types are defined once in Rust and generated to TypeScript with `ts-rs` into `packages/protocol`. Never hand-edit generated files; a CI drift check fails on any difference.
- **Every plugin reports availability.** Each plugin exposes a status command so the UI can hide what does not work on the current system.
- **Platform code is `cfg`-gated per platform module** (`linux.rs`, `windows.rs`, `unsupported.rs`), never scattered `#[cfg]` inside logic.
- **Windows are created in Rust.** `tauri.conf.json` keeps `"windows": []`; the `windows` module builds each window (decorations, transparency, shadow, size limits) in one place.
- **Capabilities are least-privilege and per window label.** Every plugin command reachable from a webview needs a `permissions/default.toml` entry, and each window label gets only the capabilities it needs.
- **Terminology.** _Native plugins_ are build-time Tauri plugins. Do not use "plugin" unqualified in identifiers or PR titles when it could mean something else.

## Documentation

- **Updates:** when user-facing behaviour, screens, or platform requirements change, update `spec/`; when a design decision is made or changed, add or amend a `docs/decisions.md` entry (D-numbers) in the same change; when structure, crate or plugin boundaries, or data flow change, update `spec/02-architecture.md`. Keep `README.md` in sync.
- **Docs are maintained alongside the code, not after it.** A PR that changes behaviour without touching the doc that describes it is incomplete.
- **Verify before release:** KDP and Kobo figures marked ⚠ in `spec/` must be checked against the live help pages before a release.
- **No hard wrapping:** see [Markdown Formatting](#markdown-formatting) above.

## Repository Layout

Cargo workspace + **Bun workspaces**, matching Waypoint, Jar and Cadence. The app's identifier is `ca.liminalhq.folio`. The authoritative design is `spec/`.

- `apps/folio` — the Tauri app: React/TypeScript frontend in `src/`, and `src-tauri/` as a thin composition root that registers plugins and wires crates together.
- `packages/chrome` — the shared React chrome (title bar, window menu, context menu, dialogs, settings shell), copied from Waypoint with Folio's `--folio-*` tokens, with no domain imports.
- `packages/ribbon` — the data-driven ribbon toolbar (tabs, groups, controls, Easy/Expert filtering, tooltips, keyboard), with no domain imports; it depends one-way on `packages/chrome` for its menus.
- `packages/protocol` — the TypeScript types `ts-rs` generates from the Rust wire types (types only, never hand-edited).
- `crates/*` — pure Rust, no `tauri` dependency: `folio-layout`, `folio-model`, `folio-package`, `folio-presets`, `folio-protocol`, `folio-session`, `folio-settings`, `folio-spell`, `folio-colour`, `folio-export`, `folio-import` and the planned `folio-jax`.
- `plugins/*` — Tauri plugins, each a Rust crate plus a `guest-js` package: `folio-books` and `folio-settings` (domain) and copies of the unpublished reusable plugins `window-tearoff` and `window-effects`.
- `spec/` — the product and architecture specification. `docs/` — decisions and developer docs. `design/` — the Claude Design prototype files, reference only. `site/` — the public documentation site (Astro Starlight, deployed to GitHub Pages).
- `scripts/` — repo tooling (`check-headers.sh` and the other checks).
- `.github/` — workflows, `dependabot.yml`, `release.yml` (changelog categories).

Items above are **planned** until they are scaffolded.

## Licence and Copyright

**REQUIREMENT:** All source code files (Rust, TypeScript, JavaScript, shell) MUST include a licence and copyright header as the first content in the file (after a shebang, for scripts).

**Header format** (Rust, TypeScript and JavaScript):

```
// Brief one-line summary of what this file does.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
```

**Rules:**

- The first line is a concise summary of the file's purpose (one sentence, no period).
- Place the header before any `use`, `import`, or `mod` statement, followed by one blank line.
- Do not add headers to generated files, configuration files (`.toml`, `.json`, `.yml`), or documentation (`.md`).
- Copied files keep their original header; their provenance is recorded in the package's `PROVENANCE.md`.
- `scripts/check-headers.sh` enforces this in CI; its exemption list covers generated and tool-scaffolded files only.

The dual licence is provided as `LICENSE-MIT` and `LICENSE-APACHE` in the repository root.

## Tauri v2

Folio uses **Tauri v2** for desktop (Linux, Windows). These are the house patterns and pitfalls carried over from Waypoint, Spindle, Cadence and Jar.

### Platform Detection

Use `@tauri-apps/plugin-os` for platform detection. `platform()` is **synchronous** and determined at compile time. Prefer a plugin's availability status over platform checks when deciding whether a feature exists.

### Tauri APIs

- Prefer Tauri plugins over web APIs when available.
- Most Tauri v2 APIs are async — use `async/await`.
- Check plugin documentation for platform-specific limitations.

### Common v1 Pitfalls (Agent Guide)

Many online resources refer to Tauri v1. In v2: `tauri` → `app` (top-level config key); `build.distDir` → `frontendDist`; `build.devPath` → `devUrl`; `tauri.allowlist` → **removed**, replaced by capabilities; `@tauri-apps/api` only exports `core`, `path`, `event` and `window`, and everything else moved to `@tauri-apps/plugin-*`.

**Permissions & capabilities (critical):** capability files live in `apps/folio/src-tauri/capabilities/`. Installing a plugin is **not** enough — permissions must be explicitly granted per plugin. Every command in a plugin's `COMMANDS` list needs a matching entry in its `permissions/default.toml`, or a webview `invoke()` silently fails the ACL check even though the Rust command exists. Rust-side calls bypass the ACL, so check the ACL first when an invoke "does nothing".

### Folio-specific pitfalls

- **Multiple webviews, one bundle.** Each window is a separate webview with its own JS heap; keep the entry bundle small and route by window label. Never assume state is shared across windows — it goes through Rust.
- **Linux webview is WebKitGTK.** Performance, `backdrop-filter`, transparency and Wayland behaviour differ from Chromium and from WebView2. Verify visual and performance work on WebKitGTK, not only in a browser.
- **Window frame and shadows differ per platform.** Windows are shadowed in code (`.shadow(true)`): the OS draws the shadow and rounded corners on Windows; Tauri's shadow is unsupported on Linux, where the chrome draws it in CSS inside a transparent margin. `data-platform` on the root element selects the tokens.
- **Logical vs physical pixels.** Cursor and window positions from Rust are physical; `screenX`/`screenY` are logical. Convert with the monitor scale factor.
- **Background work and window visibility.** Webview timers are throttled when a window is hidden; long-running work (layout, export, import) runs in Rust and pushes events.
