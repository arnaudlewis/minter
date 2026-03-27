# CLI Reference

All minter commands with flags, output, and exit codes.

---

## validate

Validate `.spec` and `.nfr` files.

```
minter validate [--deep] [<FILES>...]
```

With no arguments, reads `minter.config.json` for the specs directory and validates it.

```bash
minter validate                           # uses config (specs/ by default)
minter validate specs/user-auth.spec      # single file
minter validate specs/auth.spec specs/pay.spec  # multiple files
minter validate specs/                    # entire directory (always deep)
```

Directory validation is always deep (dependency resolution + NFR cross-validation). Single files are shallow by default.

**`--deep`** — resolve the full dependency tree and cross-validate NFR references when validating individual files.

```bash
minter validate --deep specs/payment.spec
```

```
✓ payment v2.1.0 (7 behaviors)
├── ✓ user-auth v1.0.0 (3 behaviors)
└── ✓ stripe-api v3.2.1 (8 behaviors)
    └── user-auth v1.0.0 (already shown)
2 dependencies resolved
```

Errors print to stderr with line numbers:

```
specs/broken.spec: line 12: Expected 'when' section before 'then'
```

| Exit code | Meaning |
|-----------|---------|
| `0` | All specs valid |
| `1` | One or more validation failures |

---

## watch

Watch for file changes and validate incrementally.

```
minter watch <PATH>
```

Supports single-file watching and recursive directory watching. On file change:

```
changed: payment.spec
✓ payment v2.1.0 (7 behaviors)
```

Color output (suppressed by `NO_COLOR` env var):

| Color | Meaning |
|-------|---------|
| Green | Validation success |
| Red | Failure or deleted file |
| Yellow | Changed file |
| Cyan | New file or watching banner |

Rapid saves are debounced (300ms). Press `Ctrl+C` to stop — the graph is saved before exit.

---

## format

Print the DSL grammar reference.

```
minter format <spec|nfr>
```

```bash
minter format spec   # .spec grammar
minter format nfr    # .nfr grammar
```

---

## scaffold

Generate a skeleton file on stdout.

```
minter scaffold spec
minter scaffold nfr <category>
```

Valid NFR categories: `performance`, `reliability`, `security`, `observability`, `scalability`, `cost`, `operability`.

```bash
minter scaffold spec > specs/my-feature.spec
minter scaffold nfr performance > specs/performance.nfr
```

---

## inspect

Display structured metadata for a spec or NFR file.

```
minter inspect <FILE>
```

For a `.spec` file:

```bash
minter inspect specs/user-auth.spec
```

```
user-auth v1.0.0
title: User Authentication

3 behaviors
  error_case: 1
  happy_path: 2

dependencies:
  session-store >= 1.0.0

assertion types:
  equals: 3
  is_present: 1
```

For a `.nfr` file:

```bash
minter inspect specs/performance.nfr
```

```
performance v1.0.0
title: Performance Requirements

2 constraints
  metric: 1
  rule: 1

category: performance
```

| Exit code | Meaning |
|-----------|---------|
| `0` | Success |
| `1` | File invalid or not found |

---

## guide

Print a reference guide by topic.

```
minter guide [<topic>]
```

Run without arguments to list all topics. Available topics:

| Topic | Description |
|-------|-------------|
| `methodology` | Full spec-driven development methodology |
| `workflow` | Five-phase workflow reference |
| `authoring` | Granularity, decomposition, entity format |
| `smells` | Smell detection — ambiguity, Observer Test, Swap Test |
| `nfr` | NFR design, categories, FR/NFR decision tree |
| `context` | Context management protocol for agents |
| `coverage` | Coverage tagging guide |
| `config` | Configuration reference |
| `lock` | Lock file reference |
| `ci` | CI verification reference |
| `web` | Dashboard reference |

```bash
minter guide                # list topics
minter guide methodology    # full methodology
minter guide smells         # smell detection reference
```

---

## coverage

Compute behavior coverage from `@minter` tags.

```
minter coverage [<SPEC_PATH>] [--scan <DIR>...] [--format <FORMAT>] [--verbose]
```

With no arguments, reads `minter.config.json` for specs and test directories.

```bash
minter coverage                              # uses config
minter coverage specs/                       # all specs, scan cwd
minter coverage specs/my-feature.spec        # single spec
minter coverage specs/ --scan tests/         # explicit scan dir
minter coverage specs/ --scan tests/ --scan e2e/  # multiple scan dirs
minter coverage specs/ --format json         # machine-readable output
minter coverage specs/ --verbose             # show all behaviors
```

Fully covered specs collapse to one line; specs with gaps expand:

```
Behavior Coverage
  ✓ user-auth v1.0.0  4/4 [e2e]

task-management v1.0.0
  ✓ create-task [e2e]
  ✓ list-tasks [e2e]
  ✗ complete-task uncovered
```

NFR coverage is derived automatically from the spec graph. The scanner respects `.gitignore`.

| Exit code | Meaning |
|-----------|---------|
| `0` | All behaviors covered, no tag errors |
| `1` | Uncovered behaviors or tag validation errors |

---

## graph

Visualize the dependency graph.

```
minter graph [--impacted <NAME>] [<DIR>]
```

With no arguments, reads `minter.config.json` for the specs directory.

```bash
minter graph                         # uses config
minter graph specs/                  # full graph
minter graph --impacted user-auth specs/  # reverse deps
minter graph --impacted performance specs/ # NFR reverse deps
```

Full graph output:

```
3 specs, 11 behaviors, 2 NFR categories, 5 constraints

checkout v1.0.0 (4 behaviors)
├── payment v2.1.0 (7 behaviors)
│   └── user-auth v1.0.0 (3 behaviors)
└── [nfr] performance v1.0.0 (2 constraints)
    ├── #api-response-time
    └── #db-query-time
```

`--impacted` shows reverse dependencies via BFS:

```
impacted by user-auth v1.0.0 (3 behaviors)
├── checkout v1.0.0 (4 behaviors)
└── payment v2.1.0 (7 behaviors)
```

---

## lock

Generate a `minter.lock` integrity snapshot.

```
minter lock
```

Reads `minter.config.json` for specs and test directories. Writes `minter.lock` atomically. Run this before committing whenever specs, NFR files, or tagged test files change.

See [lock-and-ci.md](lock-and-ci.md) for the lock file structure.

| Exit code | Meaning |
|-----------|---------|
| `0` | Lock file written |
| `1` | Error generating lock |

---

## ci

Verify project integrity against the lock file.

```
minter ci
```

Runs six checks:

1. Spec file integrity — every `.spec` hash matches the lock
2. NFR file integrity — every `.nfr` hash matches the lock
3. Dependency structure — all dependency edges resolve correctly
4. Test file integrity — every tagged test file hash matches the lock
5. Coverage — all behaviors covered by `@minter` tags (100%)
6. Orphan detection — no `@minter` tags reference non-existent behaviors

| Exit code | Meaning |
|-----------|---------|
| `0` | All checks pass |
| `1` | One or more checks failed |

See [lock-and-ci.md](lock-and-ci.md) for CI pipeline examples.

---

## ui

Launch the interactive dashboard.

```
minter ui [--port <PORT>] [--no-open]
```

```bash
minter ui                  # serves on :4321, opens browser
minter ui --port 8080      # custom port
minter ui --no-open        # don't open browser automatically
```

Opens `http://localhost:<port>` in your default browser. If the port is in use, tries `port+1` automatically.

Dashboard features:
- Spec cards — validation status, coverage bar, NFR badges, description
- NFR cards — constraint counts, validation status
- Slide panel — click any card for full details (behaviors, coverage, deps, errors)
- Search — filter spec cards by name
- Live updates — WebSocket pushes state on any file change (300ms debounce)
- Lock management — one-click regenerate with drift tooltip

Press `Ctrl+C` to stop the server.

See also:
- [spec-format.md](spec-format.md)
- [nfr-format.md](nfr-format.md)
- [coverage.md](coverage.md)
- [lock-and-ci.md](lock-and-ci.md)
