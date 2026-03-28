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

```bash
claude mcp add minter minter-mcp   # connect the AI spec assistant
minter ui                           # launch the live dashboard
```

Your agent knows the DSL, validation rules, and methodology — describe a feature in plain language and it writes the spec.

For the best experience, pair the MCP with the [spec engineering agent prompt](docs/spec-agent.md):

```bash
# Claude Code
mkdir -p .claude/agents && curl -o .claude/agents/spec.md \
  https://raw.githubusercontent.com/arnaudlewis/minter/main/docs/spec-agent.md
```

For other clients, copy the content of [spec-agent.md](docs/spec-agent.md) into your agent's system prompt or custom instructions.

Follow the [getting started guide](docs/getting-started.md) for the full walkthrough.

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

[Full documentation](docs/README.md)

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
