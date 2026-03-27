# NFR Cross-Reference

How `.spec` files reference NFR constraints, the containment rule, override rules, and deep validation mode.

---

## Two binding levels

NFR constraints can be bound at two levels in a spec.

### Spec-level binding

Appears in the `nfr` section at the top of the spec, after `motivation`. Applies to all behaviors in the spec.

```
nfr
  performance                    # whole-file: all constraints apply
  security#tls-required          # anchor: one specific constraint
```

Two reference forms:

| Form | Meaning |
|------|---------|
| `<category>` | Imports all constraints from the `.nfr` file |
| `<category>#<constraint>` | Targets one named constraint |

Overrides are not allowed at spec level — parse error.

### Behavior-level binding

Appears in the `nfr` section inside a behavior, after the description and before `given`. Only anchor references — whole-file references are a parse error.

```
behavior fast-checkout [happy_path]
  "Checkout completes quickly"

  nfr
    performance#api-response-time          # pin constraint, no override
    performance#api-response-time < 200ms  # pin with override (stricter)
```

---

## Containment rule

Every NFR category referenced at behavior level must also appear in the spec-level `nfr` section.

```
nfr
  performance     # declares performance is in scope

behavior my-behavior [happy_path]
  ...
  nfr
    performance#api-response-time < 200ms  # allowed: performance declared above
```

If the behavior references `reliability#uptime` but `reliability` is not in the spec-level `nfr` section, validation fails. This rule ensures the spec-level section acts as an explicit table of contents for all NFR categories in scope.

The containment rule is always checked — not just in deep mode.

---

## Override rules

A behavior-level reference may override the default threshold of a metric constraint. All four conditions must hold:

1. **Overridable** — the constraint has `overridable yes`
2. **Metric type** — only metric constraints support overrides (not rule)
3. **Operator match** — the override operator must match the original (`<` overrides `<`, not `>`)
4. **Stricter value** — the override value must be tighter than the default:
   - For `<` and `<=`: override value must be less than the original
   - For `>` and `>=`: override value must be greater than the original
   - For `==`: override value must equal the original

**Example — valid override:**

```
# performance.nfr
constraint api-response-time [metric]
  threshold < 500ms
  overridable yes
```

```
# spec
  nfr
    performance#api-response-time < 200ms   # 200 < 500, same operator — valid
```

**Example — invalid override (looser):**

```
  nfr
    performance#api-response-time < 800ms   # 800 > 500 — rejected: must be stricter
```

**Example — invalid override (wrong operator):**

```
  nfr
    performance#api-response-time > 200ms   # operator mismatch — rejected
```

---

## Value normalization

When comparing override values against default thresholds, units are normalized to a canonical number:

| Unit | Normalization |
|------|---------------|
| `ms` | Used as-is (milliseconds) |
| `s` | Multiplied by 1000 (to milliseconds) |
| `%` | Used as-is |
| `KB` | Used as-is |
| `MB` | Multiplied by 1000 (to kilobytes) |
| `GB` | Multiplied by 1000000 (to kilobytes) |
| (none) | Bare number used as-is |

Examples:
- `500ms` = 500
- `2s` = 2000
- `1MB` = 1000
- `2GB` = 2000000
- `99.9%` = 99.9

So `< 0.5s` (= 500) and `< 450ms` (= 450) — the override is stricter.

---

## Deep validation mode

Cross-reference validation only runs in deep mode:

- Directory validation is always deep
- Single-file validation requires `--deep`

```bash
minter validate specs/                    # deep (directory)
minter validate --deep specs/auth.spec    # deep (explicit)
minter validate specs/auth.spec           # shallow — cross-refs not checked
```

In deep mode, minter checks:

| Check | Description |
|-------|-------------|
| Category exists | Referenced NFR category has a corresponding `.nfr` file |
| Anchor exists | `category#constraint` matches a real constraint name |
| Overridable | Override only when `overridable yes` |
| Metric only | Override only on metric constraints |
| Operator match | Override operator matches original |
| Stricter value | Override value is tighter than default |

Rules 8 and 9 (no spec-level overrides, no behavior-level whole-file refs) are enforced at parse time — they fail even outside deep mode.

---

## NFR discovery

In deep mode, minter discovers `.nfr` files by scanning sibling directories. If your NFRs live in `specs/nfr/`, they are discovered automatically when validating `specs/`.

---

See also:
- [spec-format.md](spec-format.md) — full `.spec` grammar with NFR section details
- [nfr-format.md](nfr-format.md) — full `.nfr` grammar
- [graph-cache.md](graph-cache.md) — how the dependency graph tracks NFR references
