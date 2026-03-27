# Graph Cache

Minter maintains a graph cache at `.minter/graph.json` to speed up repeated validation runs.

---

## Location and schema

The cache lives at `.minter/graph.json` in the current working directory. It uses schema version 3. The `.minter/` directory should be in `.gitignore` — it is a local build artifact, not source.

---

## What the cache stores

For each spec and NFR file:

- SHA-256 content hash
- Spec name and version
- Behavior count (for specs)
- Constraint count (for NFR files)
- Dependency edges
- NFR category references
- Validation status

---

## When the cache is used

The cache is created automatically on first:
- Directory validation (`minter validate specs/`)
- Single-file deep validation (`minter validate --deep`)
- Watch mode (`minter watch`)

On subsequent runs, files whose SHA-256 hash matches the cached value are skipped. Only changed or new files are re-validated.

---

## Invalidation

### File change

When a `.spec` or `.nfr` file is saved, its hash changes. Minter detects the mismatch and re-validates that file on the next run.

### NFR cascade

When an `.nfr` file changes, all `.spec` files that reference that NFR category are also invalidated. This ensures that a tighter constraint in `performance.nfr` re-validates any spec bound to it.

### Stale entry pruning

When a file is deleted, its cache entry becomes stale. Minter prunes stale entries (files no longer on disk) automatically on each run.

### Schema upgrade

If the cache file was written by an older version of minter (schema version < 3), it is discarded and rebuilt from scratch. The same happens if the file is corrupted or unreadable.

---

## Atomic writes

The cache is written atomically:

1. Write new content to `.minter/graph.json.tmp`
2. Rename `.tmp` to `.minter/graph.json`

This prevents corruption if the process is interrupted mid-write.

---

## Watch mode behavior

In watch mode, the graph is maintained in memory. On file change:

1. Changed file is re-validated
2. If the file is an NFR, all specs referencing it are also re-validated
3. The in-memory graph is updated
4. The updated graph is written to `.minter/graph.json` on `Ctrl+C`

Rapid saves are debounced with a 300ms delay to prevent thrashing on auto-save.

---

## gitignore

Add `.minter/` to your `.gitignore`:

```
.minter/
```

The cache is not meaningful across machines or checkouts — it is a local optimization artifact.

---

See also:
- [cli-reference.md](cli-reference.md#validate) — `--deep` flag and validation modes
- [cli-reference.md](cli-reference.md#graph) — `minter graph` command
