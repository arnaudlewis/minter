# Getting Started

Get minter running and write your first spec in under 10 minutes.

## 1. Install

```bash
brew install arnaudlewis/tap/minter
```

This installs `minter` (CLI) and `minter-mcp` (MCP server). Verify:

```bash
minter --version
```

## 2. Connect the MCP assistant

Add `minter-mcp` to Claude Code:

```bash
claude mcp add minter minter-mcp
```

For Claude Desktop or Cursor, add to your MCP config:

```json
{
  "mcpServers": {
    "minter": { "command": "minter-mcp" }
  }
}
```

Your agent now knows the DSL grammar, all validation rules, and the spec-driven methodology. It can scaffold specs, validate syntax, assess quality, browse your project graph, and search behaviors.

For the best results, pair the MCP with the [spec engineering agent prompt](spec-agent.md):

```bash
# Claude Code
mkdir -p .claude/agents && curl -o .claude/agents/spec.md \
  https://raw.githubusercontent.com/arnaudlewis/minter/main/docs/spec-agent.md
```

For other clients, copy the content of [spec-agent.md](spec-agent.md) into your agent's system prompt or custom instructions.

See [mcp.md](mcp.md) for the full tool reference.

## 3. Write your first spec

Ask your agent to write a spec for a feature you're working on. Describe the feature in plain language — the agent handles the DSL syntax, structure, and validation. For example:

> "Create a spec for a user authentication feature. Users can register with email and password, log in, and reset their password."

The agent will scaffold the spec, fill in behaviors (happy paths, error cases, edge cases), validate it, and suggest improvements.

## 4. Launch the dashboard

From your project root:

```bash
minter ui
```

Your browser opens at `http://localhost:4321`. You'll see your spec as a card with validation status, coverage, and NFR badges. Keep the dashboard open — it updates live on every file save.

## 5. Lock and CI

Before committing, snapshot your project state:

```bash
minter lock
```

This creates `minter.lock` — a SHA-256 snapshot of every spec, NFR, and tagged test file. Commit it alongside your specs.

In CI:

```bash
minter ci
```

Six checks run: spec integrity, NFR integrity, dependency structure, test integrity, 100% coverage, and no orphan tags. Exit 1 if any fail.

See [lock-and-ci.md](lock-and-ci.md) for the full CI setup including a GitHub Actions example.

## What to read next

- [development-workflow.md](development-workflow.md) — the daily loop: spec, red tests, implement, green
- [spec-format.md](spec-format.md) — full `.spec` grammar reference
- [nfr-format.md](nfr-format.md) — full `.nfr` grammar reference
- [mcp.md](mcp.md) — all 11 MCP tools in detail
- [methodology.md](methodology.md) — five-phase workflow and principles
- [examples/README.md](../examples/README.md) — walkthrough of a complete spec project
