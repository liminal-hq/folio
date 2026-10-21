# Packaging for agent programs

Ways to put `folio-mcp` where an agent program can find it. None is published yet; the release workflow that builds the binaries for each platform has not been written (see `spec/13-agents.md`).

- `npm/folio-mcp` — the `@liminal-hq/folio-mcp` package: a small launcher that runs the binary from the platform package that matches the computer (`@liminal-hq/folio-mcp-linux-x64`, `-linux-arm64`, `-win32-x64`), so `npx @liminal-hq/folio-mcp` works anywhere. The platform packages are made from release binaries with `npm/make-platform-package.js`.
- `mcpb` — the manifest of the one-click bundle for Claude Desktop. `mcpb/build.sh` lays the bundle out from release binaries; packing it needs the `mcpb` command line tool.
- `claude-code` — a Claude Code plugin (the MCP server and a skill that explains how to make a book), listed by `.claude-plugin/marketplace.json` at the root of the repository.

The `folio-mcp install` command covers programs that read an MCP server list from a file.
