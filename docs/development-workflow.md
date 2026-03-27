# Development Workflow

The daily loop for spec-driven development with minter.

## The dashboard is always open

Start every session:

```bash
minter ui
```

Keep the dashboard open in a browser tab throughout the day. It shows the live state of your entire project — spec validation status, coverage gaps, NFR badges, and lock drift — updated on every file save via WebSocket.

The dashboard is your feedback surface. You write in your editor; the dashboard confirms the state.

## The spec-first cycle

Every feature follows five phases in order. Do not skip ahead.

### Phase 1 — Write specs

Before any code, write the spec. Scaffold a skeleton:

```bash
minter scaffold spec > specs/my-feature.spec
```

Fill in behaviors. Each behavior describes one user-observable outcome:

```
behavior login-with-email [happy_path]
  "User logs in with correct credentials and receives a session"

  given
    @user = User { email: "alice@example.com" }
    The user exists in the database

  when authenticate
    email = "alice@example.com"
    password = "secret123"

  then returns session
    assert id is_present
    assert user_id == @user.id
```

Add error cases. Add edge cases. Reference NFR constraints. Validate until green:

```bash
minter validate specs/my-feature.spec
```

The spec is complete and passes before any implementation begins.

### Phase 2 — Write documentation (optional)

Derive user-facing docs from the spec. Skip if not needed for the feature.

### Phase 3 — Write red tests

One behavior = one test. Write all tests before writing any implementation code. Every test must fail (red) before you proceed.

Tag every test with `@minter` to link it to its behavior:

```typescript
// @minter:e2e login-with-email login-invalid-password
describe("authentication", () => { /* ... */ });
```

Verify 100% coverage:

```bash
minter coverage specs/
```

Do not write implementation code until coverage is 100% and all tests are red.

### Phase 4 — Implement (TDD)

Make one test green at a time. Write unit tests alongside each implementation unit. Do not skip ahead to the next behavior until the current one is green.

### Phase 5 — All green

All e2e and unit tests pass. Run a final validate:

```bash
minter validate specs/
```

Then lock before committing:

```bash
minter lock
```

## Live feedback

The dashboard updates automatically as you work:

- Save a spec with a parse error → the card turns red with the error inline
- Fix the error → the card goes green
- Add a behavior → the behavior count updates
- Tag a test → coverage bar fills

The 300ms debounce prevents flicker on rapid saves.

## MCP assistance alongside you

The MCP server is an authoring assistant, not a CLI wrapper. While you work, your agent can:

- Scaffold a spec from a plain-English feature description
- Validate what you're writing and explain any errors
- Assess a spec for quality issues (smells, missing error cases, NFR gaps)
- Search the project graph to find related specs and behaviors

The agent already knows the methodology, DSL grammar, and all validation rules. You describe what the system should do; the agent handles the DSL.

Example workflow with agent assistance:

> "Scaffold a spec for user password reset. Include happy path, expired token error case, and rate limiting edge case. Reference the security and reliability NFR files."

The agent scaffolds, fills in behaviors, validates, and presents the result for your review.

## Lock before every commit

Before committing:

```bash
minter lock   # regenerate if specs or tests changed
minter ci     # verify — should exit 0
```

If `minter ci` fails, the lock is drifted. The dashboard header shows the drift reason with a Regenerate button. Fix what drifted, regenerate, and commit.

See [lock-and-ci.md](lock-and-ci.md) for CI pipeline setup.

## Team workflow

Specs are the contract between teams. The workflow at team scale:

1. **Spec review before implementation** — treat spec PRs the same as code PRs. Review behaviors for completeness and smells before merging.
2. **CI as the gate** — `minter ci` in every pipeline. A failing CI means specs, tests, or coverage drifted.
3. **Agent-assisted authoring** — team members use `minter-mcp` to scaffold and assess specs consistently.
4. **Dashboard as shared context** — share the dashboard URL during reviews or standups to discuss coverage and NFR gaps visually.

See also:
- [methodology.md](methodology.md) — principles behind the workflow
- [authoring-guide.md](authoring-guide.md) — how to write good behaviors
- [coverage.md](coverage.md) — tagging tests and reading coverage reports
- [lock-and-ci.md](lock-and-ci.md) — CI configuration
