# Coverage

How to tag tests with `@minter` and read coverage reports.

---

## The @minter tag

The `@minter` tag links a test to the spec behaviors it covers. It is a declaration: "this test covers this behavior." Minter scans these tags and cross-references them against the spec graph.

Tags work in `//` and `#` comment styles. Place the tag immediately above the test block it annotates.

---

## Tag formats

### Behavioral tag

```
// @minter:<type> <behavior> [<behavior>...]
```

Types: `unit`, `integration`, `e2e`

```typescript
// @minter:e2e login-with-email login-invalid-password
describe("authentication", () => { /* ... */ });

// @minter:unit validate-valid-spec
it("parses a minimal spec", () => { /* ... */ });
```

```python
# @minter:integration reject-unknown-behavior-id
def test_bad_behavior():
    ...
```

Space-separated behavior names, no commas.

### Benchmark tag

```
// @minter:benchmark #<category>#<constraint> [...]
```

```typescript
// @minter:benchmark #performance#api-response-time
bench("api latency", () => { /* ... */ });
```

Benchmark tags reference NFR constraints only. Do not mix behavior names into a benchmark tag.

---

## Test types

| Type | When to use |
|------|-------------|
| `unit` | Tests a single function or module in isolation |
| `integration` | Tests multiple components working together |
| `e2e` | Tests the full system from the user's entry point |
| `benchmark` | Measures an NFR constraint (latency, throughput) |

---

## NFR coverage derivation

Do not add NFR references to behavioral tags. NFR coverage is derived automatically from the spec graph: when a behavior that references an NFR constraint has test coverage, that constraint receives indirect (derived) coverage.

```
// @minter:e2e create-task-authenticated
// No need to add #performance#api-response-time — derived automatically
```

Only use `@minter:benchmark` to directly target NFR constraints with explicit benchmark measurements.

---

## Qualified names

If two specs share a behavior name, qualify with `spec-name/behavior-name`:

```
// @minter:unit billing-webhooks/handle-error
```

Use qualified names only when there is ambiguity. Minter reports a tag error when an unqualified name matches behaviors in multiple specs.

---

## Discovering behavior names

```bash
minter inspect specs/my-feature.spec   # lists all behavior names
minter graph specs/                    # shows the full spec tree
```

Behavior names in tags must match spec behavior names exactly.

---

## Running coverage

```bash
minter coverage                              # uses minter.config.json
minter coverage specs/                       # all specs, scan cwd
minter coverage specs/my-feature.spec        # single spec
minter coverage specs/ --scan tests/         # explicit scan directory
minter coverage specs/ --scan tests/ --scan e2e/  # multiple scan dirs
minter coverage specs/ --verbose             # show all behaviors
minter coverage specs/ --format json         # machine-readable output
```

---

## Reading the report

Fully covered specs collapse to one line. Specs with gaps expand to show individual behaviors:

```
Behavior Coverage
  ✓ user-auth v1.0.0  4/4 [e2e]

task-management v1.0.0
  ✓ create-task [e2e]
  ✓ list-tasks [e2e]
  ✓ complete-task [e2e]
  ✗ create-task-unauthenticated uncovered
  ✗ complete-nonexistent-task uncovered

NFR Coverage
  ✓ performance#api-response-time [benchmark]
  ✓ performance#throughput [benchmark]
  ✓ security#password-hashing [derived]

Summary: 7/9 behaviors covered (78%)
  e2e: 7  benchmark: 2  derived: 3
```

---

## Exit codes

| Exit code | Meaning |
|-----------|---------|
| `0` | All behaviors covered, no tag errors |
| `1` | Uncovered behaviors or tag validation errors |

---

## Common mistakes

| Mistake | Fix |
|---------|-----|
| NFR ref on behavioral tag | Remove it — NFR coverage is derived |
| Behavior name on benchmark tag | Use `#category#constraint` format |
| Missing colon | `@minter unit` (wrong) → `@minter:unit` (right) |
| Comma-separated names | `@minter:unit a, b` (wrong) → `@minter:unit a b` (right) |
| Invented behavior names | Names must exactly match spec behavior names |
| Tagging non-spec tests | Only tag tests that map to a spec behavior |

---

## Gitignore and scanning

The coverage scanner respects `.gitignore`. Files excluded from git are not scanned for tags. This means generated files, build artifacts, and vendor directories are automatically excluded.

---

See also:
- [lock-and-ci.md](lock-and-ci.md) — coverage check in CI
- [cli-reference.md](cli-reference.md#coverage) — `minter coverage` flags
- `minter guide coverage` — coverage guide in the terminal
