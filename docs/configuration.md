# Configuration

Minter uses convention-over-configuration. Most projects need no config file at all.

---

## Default conventions

| Convention | Default |
|------------|---------|
| Specs directory | `specs/` |
| Test directories | `tests/`, `benches/` |

If these directories exist in your project root and you use standard layout, no configuration is needed.

---

## minter.config.json

Optional. Place at the project root.

```json
{
  "specs": "specs/",
  "tests": ["tests/", "benches/"]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `specs` | string | Path to the specs directory |
| `tests` | array of strings | Directories to scan for `@minter` tags |

All paths are relative to the directory containing the config file.

---

## When to use config

- Non-standard directory layout (e.g., `src/specs/`)
- Additional test directories (e.g., `web/tests/`, `e2e/`)
- Monorepo with specs in a subdirectory

---

## How commands use config

| Command | Uses config for |
|---------|----------------|
| `minter validate` (no args) | Specs directory |
| `minter coverage` (no args) | Specs directory + test directories |
| `minter graph` (no args) | Specs directory |
| `minter lock` | Specs directory + test directories |
| `minter ci` | Specs directory + test directories |
| `minter ui` | Specs directory + test directories (watches all) |

Commands that accept explicit paths ignore config for those paths. `minter validate specs/auth.spec` always uses the provided path regardless of config.

---

## Monorepo layout

For a monorepo with specs per service:

```json
{
  "specs": "services/auth/specs/",
  "tests": ["services/auth/tests/", "services/auth/e2e/"]
}
```

Run minter from the service directory, or set the config path explicitly. There is one config per minter project — if you have multiple services, run minter from each service root.

---

## No config fallback

If `minter.config.json` does not exist, minter falls back to the default conventions (`specs/` and `tests/`, `benches/`). Commands that require a specs directory and cannot find one will exit with an error.

---

See also:
- [lock-and-ci.md](lock-and-ci.md) — how lock and CI use config
- [cli-reference.md](cli-reference.md) — per-command config behavior
- `minter guide config` — config reference in the terminal
