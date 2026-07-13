# MCP Deep Research

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-blue)](https://www.rust-lang.org)
[![forbid(unsafe_code)](https://img.shields.io/badge/unsafe-forbidden-red.svg)](https://doc.rust-lang.org/reference/attributes.html)

**MCP server for deep research.** Multi-source web search, content extraction, and structured synthesis delivered via the [Model Context Protocol](https://github.com/modelcontextprotocol).

## Overview

MCP Deep Research is an MCP server that provides a `deep_research` tool. Given a question, it:

1. **Searches** the web across multiple sources (DuckDuckGo)
2. **Extracts** full-page content as clean markdown
3. **Synthesizes** findings into a structured research report

## Quickstart

```bash
# Run the MCP server
cargo run
```

Then configure the server in your MCP client:

```json
{
  "mcpServers": {
    "deep-research": {
      "command": "cargo",
      "args": ["run", "--", "--transport", "stdio"]
    }
  }
}
```

## Architecture

```
Question ──→ SearchEngine ──→ ContentExtractor ──→ Synthesizer ──→ Report
                 │                    │                  │
            DuckDuckGo          HTML→Markdown       Structured
            (multi-query)       (scraper-rs)        synthesis
```

### Modules

| Module | Description |
|--------|-------------|
| `search` | Multi-source web search engine |
| `extract` | HTML-to-markdown content extraction |
| `synthesize` | Structured synthesis from multiple sources |
| `lib` | MCP server handler, types, lifecycle |

## API

### `deep_research`

**Input:**
- `question` (string) — Research question
- `max_sources` (number, optional) — Max sources to include (default: 5)
- `depth` ("quick" | "deep", optional) — Research depth

**Output:**
- `question` — Original question
- `sources` — Array of `{url, title, relevance_score, content}`
- `synthesis` — Synthesized report as markdown
- `processing_time_ms` — Total processing time

## License

MIT
