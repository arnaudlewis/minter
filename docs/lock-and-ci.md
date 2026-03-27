# Lock and CI

How `minter.lock` works, what `minter ci` checks, and how to wire it into your CI pipeline.

---

## minter.lock

`minter.lock` is an integrity snapshot of your spec project. Generate it with:

```bash
minter lock
```

### What it captures

- SHA-256 hash of every `.spec` file
- SHA-256 hash of every `.nfr` file
- SHA-256 hash of every test file containing `@minter` tags
- Behavior-to-test mapping (which tests cover which behaviors)
- Benchmark file hashes (for NFR constraint coverage)

### When to regenerate

Regenerate after modifying any `.spec`, `.nfr`, or tagged test file, after adding or removing spec or test files, and before committing. The dashboard header shows lock drift with a Regenerate button and a tooltip listing the drifted files.

The lock file is written atomically (write to `.tmp`, then rename) — it will not be corrupted if the process is interrupted.

### Commit the lock file

Commit `minter.lock` alongside your specs. This lets CI verify integrity against the exact state you committed.

Do **not** commit `.minter/` (the graph cache directory). Add it to `.gitignore`.

---

## minter ci

`minter ci` runs six checks against the lock file. All six must pass for exit 0.

```bash
minter ci
```

### The six checks

| # | Check | What it verifies |
|---|-------|------------------|
| 1 | Spec integrity | Every `.spec` file hash matches the lock |
| 2 | NFR integrity | Every `.nfr` file hash matches the lock |
| 3 | Dependency structure | All dependency edges resolve correctly |
| 4 | Test integrity | Every tagged test file hash matches the lock |
| 5 | Coverage | All spec behaviors covered by `@minter` tags (100%) |
| 6 | Orphan detection | No `@minter` tags reference non-existent behaviors |

### What each failure means

| Failure | Cause |
|---------|-------|
| Spec/NFR integrity | Files changed since last `minter lock` |
| Dependency structure | Missing or incompatible dependency version |
| Test integrity | Tagged test file changed since last lock |
| Coverage | Behaviors without `@minter`-tagged tests |
| Orphan detection | Tests reference deleted or renamed behaviors |

| Exit code | Meaning |
|-----------|---------|
| `0` | All six checks pass |
| `1` | One or more checks failed |

---

## GitHub Actions

```yaml
name: Spec Integrity

on:
  push:
    branches: [main]
  pull_request:

jobs:
  minter:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install minter
        run: |
          curl -L https://github.com/arnaudlewis/minter/releases/latest/download/minter-x86_64-unknown-linux-gnu.tar.gz \
            | tar -xz -C /usr/local/bin

      - name: Verify integrity
        run: minter ci
```

The workflow verifies that the committed `minter.lock` matches the current state of specs, NFR files, and tagged tests. If any file drifted — spec modified, test added, behavior renamed — CI fails.

---

## Typical pre-commit flow

```bash
# After editing specs or tests:
minter lock    # regenerate
minter ci      # verify — should exit 0
git add minter.lock
git commit
```

If `minter ci` fails locally, something drifted since your last lock. Fix the drift (regenerate lock, update tags, add missing coverage) before committing.

---

## Lock and the dashboard

The dashboard header shows lock status at a glance:

- **Aligned** — lock matches current files
- **Drifted** — one or more files changed since last lock; hover for a tooltip listing the drifted files
- **Regenerate** button — click to run `minter lock` from the browser

---

See also:
- [coverage.md](coverage.md) — `@minter` tag format and the coverage check
- [configuration.md](configuration.md) — configuring specs and test directories
- [cli-reference.md](cli-reference.md#lock) — `minter lock` and `minter ci` flags
