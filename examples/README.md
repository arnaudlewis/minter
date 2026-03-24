# Example: Task Management API

A small spec project showing how behavioral specs, dependencies, and NFR constraints fit together.

## What's here

```
specs/
├── user-auth.spec           # Authentication — register, login, error cases, security NFR anchors
├── task-management.spec     # Task CRUD — depends on user-auth, whole-file performance NFR
└── nfr/
    ├── performance.nfr      # Response time, throughput, and bounded query constraints
    └── security.nfr         # Password hashing and brute-force protection constraints
tests/
├── auth.test.ts             # e2e tests for user-auth behaviors
├── tasks.test.ts            # e2e tests for task-management behaviors
└── performance.bench.ts     # NFR benchmark tags for performance constraints
```

## Validate

```bash
minter validate examples/specs/
```

```
✓ performance v1.0.0 (3 constraints)
✓ security v1.0.0 (2 constraints)
✓ task-management v1.0.0 (5 behaviors)
└── ✓ user-auth v1.0.0 (4 behaviors)
1 dependency resolved
```

## Dependency graph

```bash
minter graph examples/specs/
```

```
task-management v1.0.0 (5 behaviors)
├── user-auth v1.0.0 (4 behaviors)
│   └── [nfr] security v1.0.0 (2 constraints)
│       ├── #brute-force-protection
│       └── #password-hashing
└── [nfr] performance v1.0.0 (3 constraints)
```

## Coverage

```bash
minter coverage examples/specs/ --scan examples/tests/
```

```
Behavior Coverage
  ✓ user-auth v1.0.0  4/4 [e2e]
  ✓ task-management v1.0.0  5/5 [e2e]

NFR Coverage
  ✓ performance#api-response-time [benchmark]
  ✓ performance#throughput [benchmark]
  ✓ performance#no-unbounded-queries [derived]
  ✓ security#password-hashing [derived]
  ✓ security#brute-force-protection [derived]

Summary: 9/9 behaviors covered (100%)
  e2e: 9  benchmark: 2
```

The `@minter` tags in test files link each test to the spec behaviors it covers. Benchmark tags directly target NFR constraints. Derived NFR coverage comes from the spec graph — when a behavior referencing an NFR constraint has tests, that constraint gets indirect coverage.

## What to notice

- **`user-auth.spec`** — Four behaviors covering happy path (register, login) and error cases (duplicate email, wrong password). Binds security NFR anchors at the behavior level: `security#password-hashing` on register-user, `security#brute-force-protection` on login-wrong-password. The graph expands the individual anchors.

- **`task-management.spec`** — Five behaviors across all three categories (happy_path, error_case, edge_case). Declares `depends on user-auth >= 1.0.0` and binds `performance` as a whole-file NFR — all performance constraints apply to all behaviors.

- **`nfr/performance.nfr`** — Three constraints: `api-response-time` (metric, overridable), `throughput` (metric, not overridable), and `no-unbounded-queries` (rule, not overridable). Shows both metric and rule constraint types.

- **`nfr/security.nfr`** — Two constraints: `password-hashing` (rule, critical) and `brute-force-protection` (metric, critical). Referenced via anchors from user-auth behaviors.

- **NFR binding patterns** — The example demonstrates two approaches: whole-file reference (`performance` on task-management, all constraints apply everywhere) vs anchor references (`security#password-hashing` on a specific behavior, precise binding).

- **Coverage tags** — Each test file has `// @minter:e2e <behavior>` tags above test blocks. The benchmark file uses `// @minter:benchmark #<category>#<constraint>` to directly target NFR constraints. Run `minter coverage` to see which behaviors and constraints are covered.

## Try it yourself

With the MCP server set up, ask your agent:

> "Read the specs in the examples directory. Explain the dependency graph and how NFR constraints are bound to behaviors."

Or extend the example:

> "Add a delete-task behavior to task-management.spec with both a happy path and an error case for deleting someone else's task."
