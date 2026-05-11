# rag-params-mcp

MCP (Model Context Protocol) server for [rag-params-finder](https://github.com/awdemos/rag-params-finder) semantic memory. Exposes session search, prompt search, cross-library search, and indexing tools to any MCP-compatible client.

## What It Does

This Rust MCP server wraps the rag-params-finder FastAPI backend, letting AI clients (like OpenCode, Claude Code, Cursor) recall prior context via natural language queries instead of scrolling through chat history.

## Tools

| Tool | Description |
|------|-------------|
| `search_sessions` | Semantic search over indexed session chunks (messages, errors, todos, decisions, file changes) |
| `search_prompts` | Semantic search over indexed prompt libraries |
| `cross_library_search` | Search sessions and prompts simultaneously |
| `index_session_chunk` | Index a single session chunk in real-time |
| `index_prompt` | Index a single prompt document |
| `render_prompt` | Render a prompt template with variable substitution |
| `list_sessions` | List all indexed session IDs |
| `list_libraries` | List all prompt library IDs |

## Quick Start

```bash
# Clone
git clone https://github.com/awdemos/rag-params-mcp
cd rag-params-mcp

# Build release binary
cargo build --release

# Run (connects to http://localhost:8001 by default)
./target/release/rag-params-mcp

# Or with custom backend URL
RAG_PARAMS_API_URL=http://localhost:8001 ./target/release/rag-params-mcp
```

## Configure in OpenCode

Add to `~/.config/opencode/conf/opencode.json`:

```json
{
  "mcp": {
    "rag-params-mcp": {
      "type": "local",
      "command": ["/path/to/rag-params-mcp/target/release/rag-params-mcp"],
      "enabled": true
    }
  }
}
```

Restart OpenCode. The server auto-starts on stdio when needed.

## Requirements

- Rust 1.75+
- Running rag-params-finder backend (port 8001)
- MongoDB Atlas with vector search indexes

## Architecture

```
┌─────────────┐     stdio (MCP)      ┌─────────────────┐     HTTP      ┌──────────────────┐
│   OpenCode  │ ◄──────────────────► │  rag-params-mcp │ ◄──────────► │ rag-params-finder │
│   (Client)  │   JSON-RPC 2.0       │  (This Server)  │              │   (FastAPI)       │
└─────────────┘                      └─────────────────┘              └──────────────────┘
                                                                            │
                                                                            ▼
                                                                     ┌─────────────┐
                                                                     │ MongoDB Atlas │
                                                                     │ Vector Search │
                                                                     └─────────────┘
```

## License

MIT
