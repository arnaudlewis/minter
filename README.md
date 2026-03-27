[![Latest Release](https://img.shields.io/github/v/release/arnaudlewis/minter?label=version&color=blue)](https://github.com/arnaudlewis/minter/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/arnaudlewis/minter/total?color=green)](https://github.com/arnaudlewis/minter/releases)
[![Homebrew](https://img.shields.io/badge/homebrew-arnaudlewis%2Ftap%2Fminter-orange)](https://github.com/arnaudlewis/homebrew-tap)
![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

# <picture><source media="(prefers-color-scheme: dark)" srcset="assets/logo-light.svg" /><source media="(prefers-color-scheme: light)" srcset="assets/logo-dark.svg" /><img src="assets/logo-dark.svg" alt="" width="28" height="28" /></picture> minter

The deterministic validation gate for spec-driven development.

Minter validates `.spec` and `.nfr` files — a structured DSL for defining behavioral contracts and non-functional requirements. It parses syntax, enforces semantics, resolves dependency graphs, cross-validates NFR references, and gives you an interactive dashboard and AI authoring assistant. One primitive: behaviors that depend on other behaviors. One discipline: spec first, then tests, then code.

## Install

```bash
brew install arnaudlewis/tap/minter
```

Installs both `minter` (CLI) and `minter-mcp` (MCP server).

<details>
<summary>Manual download</summary>

Download the archive for your platform from the [latest release](https://github.com/arnaudlewis/minter/releases/latest), extract it, and place `minter` and `minter-mcp` on your `PATH`. SHA-256 checksums are in `SHA256SUMS.txt`.

</details>

<details>
<summary>Build from source</summary>

```bash
cargo install minter
```

</details>

## Get Started

### 1. Launch the dashboard

```bash
minter ui
```

Opens your browser at `http://localhost:4321`. See all specs, coverage, and lock status live. The dashboard updates on every file save — keep it open while you write.

### 2. Write specs with AI assistance

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

Your agent can now scaffold specs, validate syntax, assess quality, browse the project graph, and search behaviors — and already knows the methodology.

### 3. Set up CI

```bash
minter lock    # snapshot spec + test integrity
minter ci      # verify integrity — add this to your CI pipeline
```

## Commands

| Command | Description |
|---------|-------------|
| `minter ui` | Interactive dashboard (live updates, lock management) |
| `minter validate` | Validate specs and NFR files |
| `minter watch` | Live validation on file changes |
| `minter coverage` | Behavior coverage report from `@minter` tags |
| `minter graph` | Dependency graph visualization |
| `minter lock` | Snapshot project integrity |
| `minter ci` | Verify integrity against lock file |
| `minter scaffold` | Generate skeleton `.spec` or `.nfr` files |
| `minter inspect` | Display metadata for a spec or NFR file |
| `minter format` | Print DSL grammar reference |
| `minter guide` | Development reference guides |

[Full documentation](docs/)

## Examples

The [`examples/`](examples/) directory contains a complete spec project — a task management API with authentication, CRUD behaviors, and NFR constraints.

```bash
minter validate examples/specs/
minter graph examples/specs/
minter coverage examples/specs/ --scan examples/tests/
```

See [`examples/README.md`](examples/README.md) for the full walkthrough.

## License

MIT. See [LICENSE](LICENSE).
