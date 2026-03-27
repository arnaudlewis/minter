# Minter

Rust CLI for validating .spec and .nfr files (custom DSL). Hand-written parser, SHA-256 graph caching, clap derive CLI.

## Commit Scopes

Project-specific scopes: `parser`, `validator`, `cli`, `mcp`, `graph`, `nfr`, `deps`

## Pre-Commit Checks

Before every commit, run and confirm all three pass:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Do not commit if any of these fail. Fix the issue first.

## Pull Request Merging

Always merge PRs with **squash and merge** into a single commit. The commit message must be a clear, concise summary of the entire PR using conventional commit format. Never use "Create a merge commit" or "Rebase and merge".
