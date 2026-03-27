# Methodology

The principles and five-phase workflow of spec-driven development.

---

## Principles

**Specs are the source of truth — never code.**
The spec defines what the system does. Code is an implementation of the spec. If the code and the spec disagree, the spec is right.

**TDD is mandatory.**
Every behavior gets a failing test before implementation begins. No exceptions.

**1 behavior = 1 test.**
Each behavior in a spec maps to exactly one test. Not a test suite, not a set of tests — one test. If a behavior needs more than one test, it should be split into more behaviors.

**NFRs are constraints, not guidelines.**
Non-functional requirements are enforced, not suggested. A constraint with `violation critical` is a blocker, not a nice-to-have.

**The spec is complete before any code is written.**
Phases are strictly sequential. A spec that isn't finished and validated is not ready for Phase 3.

---

## Five phases

### Phase 1 — Write specs

Use `scaffold`, `format`, and `validate` to author specs. Scaffold a skeleton, fill in behaviors, validate until green. The spec is complete before any implementation begins.

```bash
minter scaffold spec > specs/my-feature.spec
# edit the spec
minter validate specs/my-feature.spec
```

Checklist before moving to Phase 2:
- At least one `happy_path` behavior
- Error cases for each happy path
- NFR references where appropriate
- Validation passes

### Phase 2 — Write documentation (optional)

Derive user-facing docs from the spec. Skip if not needed.

### Phase 3 — Write red tests

Write one test per behavior. All tests must fail before you write any implementation code.

Tag every test with `@minter:<type>` to link it to its behavior:

```typescript
// @minter:e2e login-with-email login-invalid-password
describe("authentication", () => { /* failing tests */ });
```

Run coverage and verify completeness:

```bash
minter coverage specs/
```

Do not proceed until coverage is 100% and every test is red.

### Phase 4 — Implement (TDD)

Make one test green at a time. Unit tests are mandatory alongside implementation. Do not skip ahead to the next behavior until the current one is green end-to-end.

### Phase 5 — All green

All e2e and unit tests pass. Run a final validate to confirm spec compliance:

```bash
minter validate specs/
```

Then lock before committing:

```bash
minter lock
```

---

## One primitive

The spec format has exactly one primitive: **behaviors that depend on other behaviors**.

There is no type system — data shapes are specs whose behaviors describe what valid instances look like. There is no error catalog — error behavior is expressed directly as `error_case` behaviors. Everything is given/when/then.

One concept to learn, one concept to validate, one concept to generate tests from.

---

## Specs vs code

| Question | Answer |
|----------|--------|
| Code changed without a spec change | The code is a refactor (no behavior change) |
| Spec changed, tests haven't changed | Tests need updating — they no longer match the spec |
| New behavior added to code | Missing spec behavior — add it and write the test |
| Bug discovered | Add an `error_case` behavior that reproduces the bug, then fix it |

---

## NFR methodology

Non-functional requirements are not afterthoughts. They are defined upfront in `.nfr` files and bound to specs before implementation begins. The workflow:

1. Identify which NFR categories apply to the feature
2. Create or update the `.nfr` file for each applicable category
3. Reference the NFR constraints from the spec (spec-level and behavior-level as needed)
4. Write benchmark tests for metric constraints

See [authoring-guide.md](authoring-guide.md#nfr-design) for the FR/NFR decision tree and the seven categories.

---

See also:
- [development-workflow.md](development-workflow.md) — daily loop
- [authoring-guide.md](authoring-guide.md) — how to write good behaviors
- [coverage.md](coverage.md) — coverage tagging
- `minter guide methodology` — inline reference
