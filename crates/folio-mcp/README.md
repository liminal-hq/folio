# folio-mcp

A [Model Context Protocol](https://modelcontextprotocol.io) server for [Folio](https://github.com/liminal-hq/folio), the local-first book layout app. It lets an AI agent make a book, import a manuscript, edit pages, check them, look at them and export them.

Run it over stdio: `folio-mcp`. The agent's tools are listed by the server itself; the intents `apply_intents` accepts are described by the `folio://schema/intent` resource.
