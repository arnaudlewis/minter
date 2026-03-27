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

## 2. Launch the dashboard

From your project root, start the dashboard:

```bash
minter ui
```

Your browser opens at `http://localhost:4321`. The dashboard is empty — you have no specs yet. Keep it open. It updates on every file save.

## 3. Create your first spec

Generate a skeleton:

```bash
minter scaffold spec > specs/my-feature.spec
```

Open `specs/my-feature.spec` in your editor. You'll see:

```
spec my-feature v0.1.0
title "My Feature"

description
  Describe what this feature does.

motivation
  Explain why this feature is needed.

nfr
  performance

behavior do-something [happy_path]
  "The system does something successfully"

  given
    The system is in a valid state

  when perform-action

  then returns result
    assert status == "success"
```

Edit the spec to describe a real behavior. Replace the placeholder text with your feature's actual inputs, outputs, and preconditions. See [spec-format.md](spec-format.md) for the full grammar.

When you save, the dashboard card updates immediately.

## 4. Validate your spec

```bash
minter validate specs/my-feature.spec
```

```
✓ my-feature v0.1.0 (1 behavior)
```

Errors print to stderr with line numbers and fix suggestions:

```
specs/my-feature.spec: line 8: Expected 'title' after spec header
```

Fix errors until validation passes. Then add more behaviors — at least one `error_case` for each `happy_path`.

## 5. Add NFR constraints (optional)

Generate a performance NFR:

```bash
minter scaffold nfr performance > specs/performance.nfr
```

Reference it from your spec:

```
nfr
  performance
```

Validate with deep mode to cross-check all NFR references:

```bash
minter validate --deep specs/my-feature.spec
```

## 6. Set up the MCP assistant

Add `minter-mcp` to Claude Code:

```bash
claude mcp add minter minter-mcp
```

Your agent now knows the DSL grammar, all validation rules, and the spec-driven methodology. Ask it to:

- Scaffold and fill in a new spec from a feature description
- Validate a spec and explain the errors
- Assess a spec for quality issues
- Browse your project with `list_specs` and `search`

See [mcp.md](mcp.md) for full setup and all 11 tools.

## 7. Lock and CI

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
- [mcp.md](mcp.md) — MCP setup and all 11 agent tools
- [methodology.md](methodology.md) — five-phase workflow and principles
- [examples/README.md](../examples/README.md) — walkthrough of a complete spec project
