# 13 · Working with an AI agent

Folio can be driven by an AI agent through a [Model Context Protocol](https://modelcontextprotocol.io) server, `folio-mcp` (D-45). An agent can make a book, bring a manuscript in, change pages, look at them, run preflight and export, and the person can have the same book open while it does: they watch it change and can undo what the agent did.

## Two ways in, one set of tools

- **Live.** While Folio is running it listens on a local socket (a Unix socket in the runtime folder with mode 0600, a named pipe on Windows closed to other computers). `folio-mcp` finds it and relays the agent's standard input and output to it, so the agent works on the books the person has open: the same sessions, the same undo history, the same events (D-02). The window follows the agent's changes as it follows its own, and says so in the status bar for a few seconds.
- **Headless.** With no app running, `folio-mcp` hosts books of its own through the same engine (`folio-engine`, D-44). A book opened this way is locked with a small `<book>.folio.lock` beside it, held by the operating system and let go of if the program dies, so the app and an agent never both write one file. A program that finds the lock gets "already open in another window or program".

`--mode live` fails if Folio is not running, `--mode headless` never uses the app, and the default `auto` uses the app if it is there. `FOLIO_AGENTS=off` keeps the app's socket closed, and `FOLIO_AGENT_ENDPOINT` names another socket (two copies of the app, a test).

## What an agent can do

| Tool | What it does |
|---|---|
| `list_kinds` | The kinds of book and the trim sizes each comes in |
| `new_book`, `open_book`, `list_books`, `close_book` | Make a book in memory, open a `.folio`, list or close open books |
| `get_book`, `get_spread` | The book's shape (spreads and their frames), and one spread in full with its layout |
| `render_page` | A PNG of one page, as drawn, so the agent can look at its work |
| `preflight`, `story_chapters` | The problems a printer would find; the chapters of the story |
| `book_report` | What one of the app's Book tools finds (`{"tool": "pictures"}`: every picture with its pixels per inch and status; `{"tool": "checklist"}`: what a store will still ask for; `{"tool": "royalty", …}`: print cost and royalties; `{"tool": "punctuation"}` and `{"tool": "styles"}`: what the typesetting tools would tidy; `{"tool": "contents"}` and `{"tool": "copyright", "fields": {…}}`: the chapters with their pages and the words of the copyright page), the same report the tools window shows (D-49) |
| `apply_intents` | Any change the app can make: a list of intents applied together as one undo step, or not at all |
| `undo`, `redo` | Step through history |
| `import_manuscript`, `import_picture` | Bring in Markdown, plain text, Word, OpenDocument or Rich Text as the story; bring in a picture |
| `save_book`, `export` | Save a `.folio`; make the sharing PDF, KDP files or EPUBs beside it |

`apply_intents` takes the same intents the window sends (spec 02, D-02), so there is no tool per intent. They are described by two resources: `folio://guide/intents`, the ones used most with an example each, and `folio://schema/intent`, the JSON Schema of every intent, made from the Rust types. A batch is checked before anything is applied, and a refused intent takes back those before it.

## Safety

- The socket is reachable only by the person who runs Folio. An agent can do what that person can do with Folio, which is edit and export books and read files it is given the path of; it cannot run programs or reach the network through Folio.
- Every agent change is one undo step the person can take back, and is marked as the agent's in the event the window hears.
- Tools that overwrite or discard (`save_book`, `close_book`) say so in their annotations, so a harness can ask first.
- Confining the paths an agent may name to a chosen folder is not built yet.

## Adding Folio to an agent program

`folio-mcp install <program> [--print]` adds Folio to the list of MCP servers of Claude Code, Claude Desktop, Cursor, VS Code, Windsurf, Codex or Gemini, changing only its own entry and keeping a copy of the file as it was. `folio-mcp doctor` says what is and is not set up. The paths each program keeps its list at are those it documents ⚠ and are checked against its documentation before a release.

The app has a **Connect an agent…** item in its menu (Help) that says whether agents can reach the open books and adds Folio to any of those programs with one click. It finds `folio-mcp` beside the app (an AppImage runs from a mount that is gone once Folio quits, so it keeps a copy in `$XDG_DATA_HOME/ca.liminalhq.folio/bin` and gives agent programs that path): the bundle carries it as a sidecar (`bun run tauri:bundle`, which builds it with `scripts/prepare-sidecar.sh` and merges `tauri.bundle.conf.json`, kept out of `tauri dev` because Tauri insists the sidecar exists).

Beyond that, the files in `packaging/` make the other ways in: an npm package that runs the right binary for the platform (`npx @liminal-hq/folio-mcp`), a one-click bundle for Claude Desktop (`.mcpb`), and a Claude Code plugin with a skill that explains the workflow. Nothing is published yet: the crate is built to be publishable but is not on crates.io. The release workflow builds the binaries, packs the npm packages and the `.mcpb` bundle and attaches them to the GitHub Release; it publishes to npm only when the `PUBLISH_MCP_TO_NPM` repository variable is `true`.
