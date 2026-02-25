/// Print the spec-driven development methodology reference and exit.
pub fn run_explain() -> i32 {
    print_methodology();
    0
}

fn print_methodology() {
    println!(
        "\
Spec-Driven Development Methodology
====================================

Specs are the source of truth for every feature. Each spec declares
behaviors — the atomic unit of work. 1 behavior maps to 1 test. Every
behavior belongs to a category: happy_path, error_case, or edge_case.


Non-Functional Requirements (NFRs)
-----------------------------------

NFRs define non-functional quality attributes as constraints. Each
constraint is either a metric (measurable threshold) or a rule
(binary pass/fail policy).

There are seven NFR categories:
  performance, reliability, security, observability,
  scalability, cost, operability


Cross-Reference Binding
-----------------------

Specs reference NFR constraints via an `nfr` section. There are two
binding levels:

  spec-level    — applies the constraint to all behaviors in the spec
  behavior-level — pins a specific anchor constraint to one behavior

References come in three forms:

  category                           whole-file reference
  category#constraint                anchor reference
  category#constraint operator value override (behavior-level only)


Whole-File vs Anchor References
-------------------------------

A whole-file reference (just `category`) imports every constraint from
the corresponding .nfr file. An anchor reference (`category#constraint`)
targets a single named constraint via the `#` anchor syntax.


Containment Rule
----------------

Every category referenced at behavior-level must also appear in the
spec-level `nfr` section. This containment rule ensures spec-level
declarations act as a table of contents for the NFR categories in scope.


Override Rules
--------------

A behavior-level reference may override the default threshold of a
metric constraint. Overrides are only allowed when:

  - The constraint is marked `overridable yes`
  - The constraint is a metric (not a rule)
  - The override operator matches the original threshold operator
  - The override value is stricter than the default


Test Generation
---------------

Each NFR reference in a spec emits a test obligation. The validate
command checks that all references resolve to real constraints and
that overrides satisfy the rules above. Test runners generate one
test per behavior per bound constraint."
    );
}
