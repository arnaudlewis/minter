# Contributing to minter

## Development setup

```bash
git clone https://github.com/arnaudlewis/minter.git
cd minter
cargo build
```

**Requirements:** Rust 1.85+ (edition 2024)

## How minter is built

Minter is built with its own methodology. The `specs/` directory contains behavioral specifications that describe every feature and validation rule. These specs are the source of truth — code implements what the specs define.

**Workflow:**

1. **Spec first** — Write or update a `.spec` file describing the behavior
2. **Validate** — `minter validate specs/` to confirm the spec is well-formed
3. **Red tests** — Write tests that cover the spec behaviors (tagged with `@minter`)
4. **Implement** — Write code until tests pass
5. **All green** — `cargo test` passes, coverage tags match spec behaviors

If you're adding a feature or fixing a bug, start by looking at the relevant spec in `specs/`. If no spec covers the change, write one first.

## Pre-commit checks

Before submitting a PR, all three must pass:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Commit conventions

[Conventional Commits](https://www.conventionalcommits.org/) with scopes: `parser`, `validator`, `cli`, `mcp`, `graph`, `nfr`, `deps`.

```
feat(parser): support multi-line descriptions
fix(validator): reject duplicate alias names
```

## Reporting issues

Use [GitHub Issues](https://github.com/arnaudlewis/minter/issues). Include your `minter --version`, OS, steps to reproduce, and relevant `.spec`/`.nfr` content if applicable.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
