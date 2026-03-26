use super::*;

// ── Helpers ─────────────────────────────────────────────

const MINIMAL: &str = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";

fn spec_with_header(name: &str, version: &str, title: &str) -> String {
    format!(
        "\
spec {name} {version}
title \"{title}\"

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
"
    )
}

fn spec_with_behavior(behavior_block: &str) -> String {
    format!(
        "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.

{behavior_block}
"
    )
}

fn spec_with_deps(deps: &str) -> String {
    format!(
        "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

{deps}
"
    )
}

// ═══════════════════════════════════════════════════════════════
// Header parsing (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-spec-declaration
#[test]
fn parse_spec_declaration() {
    let spec = parse(MINIMAL).unwrap();
    assert_eq!(spec.name, "test-spec");
    assert_eq!(spec.version, "1.0.0");
}

/// dsl-format.spec: parse-title
#[test]
fn parse_title() {
    let input = spec_with_header("test-spec", "v2.3.1", "A Descriptive Title");
    let spec = parse(&input).unwrap();
    assert_eq!(spec.title, "A Descriptive Title");
    assert_eq!(spec.version, "2.3.1");
}

/// dsl-format.spec: parse-description-block
#[test]
fn parse_description_block() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  Line one.
  Line two.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let spec = parse(input).unwrap();
    assert!(spec.description.contains("Line one"));
    assert!(spec.description.contains("Line two"));
}

/// dsl-format.spec: parse-motivation-block
#[test]
fn parse_motivation_block() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Why this exists.
  Another reason.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let spec = parse(input).unwrap();
    assert!(spec.motivation.contains("Why this exists"));
    assert!(spec.motivation.contains("Another reason"));
}

// ═══════════════════════════════════════════════════════════════
// Behavior blocks (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-behavior-declaration
#[test]
fn parse_behavior_declaration() {
    let spec = parse(MINIMAL).unwrap();
    assert_eq!(spec.behaviors.len(), 1);
    assert_eq!(spec.behaviors[0].name, "do-thing");
    assert_eq!(spec.behaviors[0].description, "Do it");
}

/// dsl-format.spec: parse-category-happy-path
#[test]
fn parse_category_happy_path() {
    let spec = parse(MINIMAL).unwrap();
    assert_eq!(spec.behaviors[0].category, BehaviorCategory::HappyPath);
}

/// dsl-format.spec: parse-category-error-case
#[test]
fn parse_category_error_case() {
    let input = spec_with_behavior(
        "\
behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

behavior fail-thing [error_case]
  \"Fail\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"",
    );
    let spec = parse(&input).unwrap();
    assert_eq!(spec.behaviors[1].category, BehaviorCategory::ErrorCase);
}

/// dsl-format.spec: parse-category-edge-case
#[test]
fn parse_category_edge_case() {
    let input = spec_with_behavior(
        "\
behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

behavior weird-thing [edge_case]
  \"Edge\"

  given
    Unusual conditions

  when act

  then emits stdout
    assert output contains \"handled\"",
    );
    let spec = parse(&input).unwrap();
    assert_eq!(spec.behaviors[1].category, BehaviorCategory::EdgeCase);
}

// ═══════════════════════════════════════════════════════════════
// Given section (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-given-prose
#[test]
fn parse_given_prose() {
    let spec = parse(MINIMAL).unwrap();
    assert_eq!(spec.behaviors[0].preconditions.len(), 1);
    match &spec.behaviors[0].preconditions[0] {
        Precondition::Prose(text) => assert_eq!(text, "Ready"),
        other => panic!("Expected Prose, got {other:?}"),
    }
}

/// dsl-format.spec: parse-given-alias-declaration
#[test]
fn parse_given_alias_declaration() {
    let input = spec_with_behavior(
        "\
behavior with-alias [happy_path]
  \"Has alias\"

  given
    @the_user = User { id: \"550e8400\", name: \"Alice\" }

  when act

  then emits stdout
    assert output contains \"done\"",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].preconditions[0] {
        Precondition::Alias {
            name,
            entity,
            properties,
        } => {
            assert_eq!(name, "the_user");
            assert_eq!(entity, "User");
            assert_eq!(properties.len(), 2);
            assert_eq!(properties[0], ("id".to_string(), "550e8400".to_string()));
            assert_eq!(properties[1], ("name".to_string(), "Alice".to_string()));
        }
        other => panic!("Expected Alias, got {other:?}"),
    }
}

/// dsl-format.spec: parse-given-multiple-preconditions
#[test]
fn parse_given_multiple_preconditions() {
    let input = spec_with_behavior(
        "\
behavior multi [happy_path]
  \"Multiple preconditions\"

  given
    The system is ready
    @the_user = User { id: \"123\" }

  when act

  then emits stdout
    assert output contains \"done\"",
    );
    let spec = parse(&input).unwrap();
    assert_eq!(spec.behaviors[0].preconditions.len(), 2);
    assert!(matches!(
        &spec.behaviors[0].preconditions[0],
        Precondition::Prose(_)
    ));
    assert!(matches!(
        &spec.behaviors[0].preconditions[1],
        Precondition::Alias { .. }
    ));
}

// ═══════════════════════════════════════════════════════════════
// When section (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-when-action
#[test]
fn parse_when_action() {
    let input = spec_with_behavior(
        "\
behavior with-action [happy_path]
  \"Named action\"

  given
    Ready

  when create_item

  then emits stdout
    assert output contains \"done\"",
    );
    let spec = parse(&input).unwrap();
    assert_eq!(spec.behaviors[0].action.name, "create_item");
}

/// dsl-format.spec: parse-when-inputs
#[test]
fn parse_when_inputs() {
    let input = spec_with_behavior(
        "\
behavior with-inputs [happy_path]
  \"Typed inputs\"

  given
    Ready

  when create_item
    name = \"test\"
    count = 42

  then emits stdout
    assert output contains \"done\"",
    );
    let spec = parse(&input).unwrap();
    let inputs = &spec.behaviors[0].action.inputs;
    assert_eq!(inputs.len(), 2);
    match &inputs[0] {
        ActionInput::Value { name, value } => {
            assert_eq!(name, "name");
            assert_eq!(value, "test");
        }
        other => panic!("Expected Value, got {other:?}"),
    }
    match &inputs[1] {
        ActionInput::Value { name, value } => {
            assert_eq!(name, "count");
            assert_eq!(value, "42");
        }
        other => panic!("Expected Value, got {other:?}"),
    }
}

/// dsl-format.spec: parse-when-alias-reference
#[test]
fn parse_when_alias_reference() {
    let input = spec_with_behavior(
        "\
behavior with-ref [happy_path]
  \"Alias ref in when\"

  given
    @the_user = User { id: \"550e8400\" }

  when act
    user_id = @the_user.id

  then emits stdout
    assert output contains \"done\"",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].action.inputs[0] {
        ActionInput::AliasRef { name, alias, field } => {
            assert_eq!(name, "user_id");
            assert_eq!(alias, "the_user");
            assert_eq!(field, "id");
        }
        other => panic!("Expected AliasRef, got {other:?}"),
    }
}

// ═══════════════════════════════════════════════════════════════
// Then section (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-then-returns
#[test]
fn parse_then_returns() {
    let input = spec_with_behavior(
        "\
behavior with-returns [happy_path]
  \"Returns\"

  given
    Ready

  when act

  then returns created item
    assert id is_present
    assert name == \"test\"",
    );
    let spec = parse(&input).unwrap();
    let pc = &spec.behaviors[0].postconditions[0];
    assert_eq!(
        pc.kind,
        PostconditionKind::Returns("created item".to_string())
    );
    assert_eq!(pc.assertions.len(), 2);
}

/// dsl-format.spec: parse-then-emits
#[test]
fn parse_then_emits() {
    let spec = parse(MINIMAL).unwrap();
    let pc = &spec.behaviors[0].postconditions[0];
    assert_eq!(pc.kind, PostconditionKind::Emits("stdout".to_string()));
}

/// dsl-format.spec: parse-then-emits-process-exit
#[test]
fn parse_then_emits_process_exit() {
    let input = spec_with_behavior(
        "\
behavior with-exit [happy_path]
  \"Exit code\"

  given
    Ready

  when act

  then emits process_exit
    assert code == 0",
    );
    let spec = parse(&input).unwrap();
    assert_eq!(
        spec.behaviors[0].postconditions[0].kind,
        PostconditionKind::Emits("process_exit".to_string())
    );
}

/// dsl-format.spec: parse-then-side-effect
#[test]
fn parse_then_side_effect() {
    let input = spec_with_behavior(
        "\
behavior with-side-effect [happy_path]
  \"Side effect\"

  given
    Ready

  when act

  then side_effect
    assert Note entity created with title == \"test\"",
    );
    let spec = parse(&input).unwrap();
    assert_eq!(
        spec.behaviors[0].postconditions[0].kind,
        PostconditionKind::SideEffect
    );
    assert_eq!(spec.behaviors[0].postconditions[0].assertions.len(), 1);
}

/// dsl-format.spec: parse-multiple-then-blocks
#[test]
fn parse_multiple_then_blocks() {
    let input = spec_with_behavior(
        "\
behavior multi-then [happy_path]
  \"Multiple then\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

  then emits process_exit
    assert code == 0",
    );
    let spec = parse(&input).unwrap();
    assert_eq!(spec.behaviors[0].postconditions.len(), 2);
}

// ═══════════════════════════════════════════════════════════════
// Assertions (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-assert-equals
#[test]
fn parse_assert_equals() {
    let input = spec_with_behavior(
        "\
behavior with-eq [happy_path]
  \"Equals\"

  given
    Ready

  when act

  then returns result
    assert name == \"test\"",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].postconditions[0].assertions[0] {
        Assertion::Equals { field, value } => {
            assert_eq!(field, "name");
            assert_eq!(value, "test");
        }
        other => panic!("Expected Equals, got {other:?}"),
    }
}

/// dsl-format.spec: parse-assert-is-present
#[test]
fn parse_assert_is_present() {
    let input = spec_with_behavior(
        "\
behavior with-present [happy_path]
  \"IsPresent\"

  given
    Ready

  when act

  then returns result
    assert id is_present",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].postconditions[0].assertions[0] {
        Assertion::IsPresent { field } => assert_eq!(field, "id"),
        other => panic!("Expected IsPresent, got {other:?}"),
    }
}

/// dsl-format.spec: parse-assert-contains
#[test]
fn parse_assert_contains() {
    let spec = parse(MINIMAL).unwrap();
    match &spec.behaviors[0].postconditions[0].assertions[0] {
        Assertion::Contains { field, value } => {
            assert_eq!(field, "output");
            assert_eq!(value, "done");
        }
        other => panic!("Expected Contains, got {other:?}"),
    }
}

/// dsl-format.spec: parse-assert-in-range
#[test]
fn parse_assert_in_range() {
    let input = spec_with_behavior(
        "\
behavior with-range [happy_path]
  \"Range\"

  given
    Ready

  when act

  then returns result
    assert count in_range 1..100",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].postconditions[0].assertions[0] {
        Assertion::InRange { field, min, max } => {
            assert_eq!(field, "count");
            assert_eq!(min, "1");
            assert_eq!(max, "100");
        }
        other => panic!("Expected InRange, got {other:?}"),
    }
}

/// dsl-format.spec: parse-assert-matches-pattern
#[test]
fn parse_assert_matches_pattern() {
    let input = spec_with_behavior(
        "\
behavior with-pattern [happy_path]
  \"Pattern\"

  given
    Ready

  when act

  then returns result
    assert email matches_pattern \"^.+@.+$\"",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].postconditions[0].assertions[0] {
        Assertion::MatchesPattern { field, pattern } => {
            assert_eq!(field, "email");
            assert_eq!(pattern, "^.+@.+$");
        }
        other => panic!("Expected MatchesPattern, got {other:?}"),
    }
}

/// dsl-format.spec: parse-assert-equals-ref
#[test]
fn parse_assert_equals_ref() {
    let input = spec_with_behavior(
        "\
behavior with-ref [happy_path]
  \"EqualsRef\"

  given
    @the_user = User { id: \"123\" }

  when act

  then returns result
    assert created_by == @the_user.id",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].postconditions[0].assertions[0] {
        Assertion::EqualsRef {
            field,
            alias,
            alias_field,
        } => {
            assert_eq!(field, "created_by");
            assert_eq!(alias, "the_user");
            assert_eq!(alias_field, "id");
        }
        other => panic!("Expected EqualsRef, got {other:?}"),
    }
}

/// dsl-format.spec: parse-assert-greater-or-equal
#[test]
fn parse_assert_greater_or_equal() {
    let input = spec_with_behavior(
        "\
behavior with-gte [happy_path]
  \"GTE\"

  given
    Ready

  when act

  then returns result
    assert count >= 2",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].postconditions[0].assertions[0] {
        Assertion::GreaterOrEqual { field, value } => {
            assert_eq!(field, "count");
            assert_eq!(value, "2");
        }
        other => panic!("Expected GreaterOrEqual, got {other:?}"),
    }
}

// ═══════════════════════════════════════════════════════════════
// Multi-word field assertions
// ═══════════════════════════════════════════════════════════════

#[test]
fn parse_assert_multi_word_field() {
    let input = spec_with_behavior(
        "\
behavior with-multi-field [happy_path]
  \"Multi-word field\"

  given
    Ready

  when act

  then
    assert behavior name == \"do-thing\"
    assert behavior category == \"happy_path\"",
    );
    let spec = parse(&input).unwrap();
    match &spec.behaviors[0].postconditions[0].assertions[0] {
        Assertion::Equals { field, value } => {
            assert_eq!(field, "behavior name");
            assert_eq!(value, "do-thing");
        }
        other => panic!("Expected Equals with multi-word field, got {other:?}"),
    }
    match &spec.behaviors[0].postconditions[0].assertions[1] {
        Assertion::Equals { field, value } => {
            assert_eq!(field, "behavior category");
            assert_eq!(value, "happy_path");
        }
        other => panic!("Expected Equals with multi-word field, got {other:?}"),
    }
}

/// Prose assertions (no known operator) should parse as Prose, not error
#[test]
fn parse_assert_prose_no_operator() {
    let input = spec_with_behavior(
        "\
behavior with-prose-assert [happy_path]
  \"Prose assertions\"

  given
    Ready

  when act

  then
    assert assertions are captured
    assert all preconditions are captured in order",
    );
    let spec = parse(&input).unwrap();
    let assertions = &spec.behaviors[0].postconditions[0].assertions;
    assert_eq!(assertions.len(), 2);
    match &assertions[0] {
        Assertion::Prose(text) => assert_eq!(text, "assertions are captured"),
        other => panic!("Expected Prose, got {other:?}"),
    }
    match &assertions[1] {
        Assertion::Prose(text) => assert_eq!(text, "all preconditions are captured in order"),
        other => panic!("Expected Prose, got {other:?}"),
    }
}

// ═══════════════════════════════════════════════════════════════
// Dependencies (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-depends-on
#[test]
fn parse_depends_on() {
    let input = spec_with_deps("depends on user-auth >= 1.0.0");
    let spec = parse(&input).unwrap();
    assert_eq!(spec.dependencies.len(), 1);
    assert_eq!(spec.dependencies[0].spec_name, "user-auth");
    assert_eq!(spec.dependencies[0].version_constraint, "1.0.0");
}

/// dsl-format.spec: parse-multiple-dependencies
#[test]
fn parse_multiple_dependencies() {
    let input = spec_with_deps("depends on user-auth >= 1.0.0\ndepends on billing >= 2.0.0");
    let spec = parse(&input).unwrap();
    assert_eq!(spec.dependencies.len(), 2);
    assert_eq!(spec.dependencies[0].spec_name, "user-auth");
    assert_eq!(spec.dependencies[1].spec_name, "billing");
}

// ═══════════════════════════════════════════════════════════════
// Comments (# lines are ignored)
// ═══════════════════════════════════════════════════════════════

#[test]
fn comments_are_ignored_in_header() {
    let input = "\
spec test-spec v1.0.0
# this is a comment
title \"Test\"

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let spec = parse(input).unwrap();
    assert_eq!(spec.name, "test-spec");
    assert_eq!(spec.title, "Test");
}

#[test]
fn comments_are_ignored_between_behaviors() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.

# first group

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

# second group

behavior other-thing [happy_path]
  \"Do other\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"ok\"
";
    let spec = parse(input).unwrap();
    assert_eq!(spec.behaviors.len(), 2);
    assert_eq!(spec.behaviors[0].name, "do-thing");
    assert_eq!(spec.behaviors[1].name, "other-thing");
}

#[test]
fn comments_are_ignored_before_deps() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

# dependencies
depends on user-auth >= 1.0.0
";
    let spec = parse(input).unwrap();
    assert_eq!(spec.dependencies.len(), 1);
    assert_eq!(spec.dependencies[0].spec_name, "user-auth");
}

// ═══════════════════════════════════════════════════════════════
// No --- separators (must work without them)
// ═══════════════════════════════════════════════════════════════

#[test]
fn reject_separator_as_unknown_keyword() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.

---

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    // --- should be treated as unknown content and rejected
    let errors = parse(input).unwrap_err();
    assert!(!errors.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// Structure (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: ignore-blank-lines
#[test]
fn ignore_blank_lines() {
    let input = "\
spec test-spec v1.0.0

title \"Test\"


description
  A test.


motivation
  Testing.


behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

";
    let spec = parse(input).unwrap();
    assert_eq!(spec.name, "test-spec");
    assert_eq!(spec.behaviors.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// Error cases (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: reject-behavior-without-given
#[test]
fn reject_behavior_without_given() {
    let input = spec_with_behavior(
        "\
behavior bad [happy_path]
  \"No given\"

  when act

  then emits stdout
    assert output contains \"done\"",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = errors[0].message.to_lowercase();
    assert!(
        msg.contains("given"),
        "Expected mention of 'given', got: {msg}"
    );
}

/// dsl-format.spec: reject-behavior-without-when
#[test]
fn reject_behavior_without_when() {
    let input = spec_with_behavior(
        "\
behavior bad [happy_path]
  \"No when\"

  given
    Ready

  then emits stdout
    assert output contains \"done\"",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = errors[0].message.to_lowercase();
    assert!(
        msg.contains("when"),
        "Expected mention of 'when', got: {msg}"
    );
}

/// dsl-format.spec: reject-behavior-without-then
#[test]
fn reject_behavior_without_then() {
    let input = spec_with_behavior(
        "\
behavior bad [happy_path]
  \"No then\"

  given
    Ready

  when act
",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = errors[0].message.to_lowercase();
    assert!(
        msg.contains("then"),
        "Expected mention of 'then', got: {msg}"
    );
}

/// dsl-format.spec: reject-wrong-section-order
#[test]
fn reject_wrong_section_order() {
    let input = spec_with_behavior(
        "\
behavior bad [happy_path]
  \"Wrong order\"

  given
    Ready

  then emits stdout
    assert output contains \"done\"

  when act",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
}

/// dsl-format.spec: reject-assert-without-field
#[test]
fn reject_assert_without_field() {
    let input = spec_with_behavior(
        "\
behavior bad [happy_path]
  \"No field\"

  given
    Ready

  when act

  then emits stdout
    assert == \"test\"",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
}

/// dsl-format.spec: reject-unknown-assertion-operator
#[test]
fn reject_unknown_assertion_operator() {
    let input = spec_with_behavior(
        "\
behavior bad [happy_path]
  \"Unknown op\"

  given
    Ready

  when act

  then emits stdout
    assert name frobnicates \"test\"",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = &errors[0].message;
    assert!(
        msg.contains("frobnicates") || msg.contains("operator"),
        "Expected mention of bad operator, got: {msg}"
    );
}

/// dsl-format.spec: reject-alias-without-entity
#[test]
fn reject_alias_without_entity() {
    let input = spec_with_behavior(
        "\
behavior bad [happy_path]
  \"No entity\"

  given
    @my_alias = { id: \"123\" }

  when act

  then emits stdout
    assert output contains \"done\"",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
}

/// dsl-format.spec: reject-malformed-alias-reference
#[test]
fn reject_malformed_alias_reference() {
    let input = spec_with_behavior(
        "\
behavior bad [happy_path]
  \"Bad ref\"

  given
    The system is ready

  when act
    user_id = @

  then emits stdout
    assert output contains \"done\"",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
}

/// dsl-format.spec: reject-depends-on-without-version
#[test]
fn reject_depends_on_without_version() {
    let input = spec_with_deps("depends on user-auth");
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = &errors[0].message;
    assert!(
        msg.contains("version") || msg.contains("user-auth"),
        "Expected mention of missing version, got: {msg}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Edge cases from validate-command.spec
// ═══════════════════════════════════════════════════════════════

#[test]
fn reject_empty_input() {
    let errors = parse("").unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn reject_missing_spec_header() {
    let errors = parse("title \"Test\"").unwrap_err();
    assert!(!errors.is_empty());
    let msg = errors[0].message.to_lowercase();
    assert!(
        msg.contains("spec"),
        "Expected mention of 'spec', got: {msg}"
    );
}

#[test]
fn reject_missing_title() {
    let input = "\
spec test-spec v1.0.0

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let errors = parse(input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = errors[0].message.to_lowercase();
    assert!(
        msg.contains("title"),
        "Expected mention of 'title', got: {msg}"
    );
}

#[test]
fn reject_missing_description() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let errors = parse(input).unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn reject_missing_motivation() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let errors = parse(input).unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn reject_no_behaviors() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.
";
    let errors = parse(input).unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn reject_unknown_keyword() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"
frobnicate something

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let errors = parse(input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = &errors[0].message;
    assert!(
        msg.contains("frobnicate") || msg.contains("keyword"),
        "Expected mention of unknown keyword, got: {msg}"
    );
}

#[test]
fn reject_unclosed_quote() {
    let input = spec_with_header("test-spec", "v1.0.0", "Unclosed Title");
    let input = input.replace("\"Unclosed Title\"", "\"Unclosed Title");
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn reject_invalid_category() {
    let input = spec_with_behavior(
        "\
behavior bad [unknown_category]
  \"Bad category\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = &errors[0].message;
    assert!(
        msg.contains("unknown_category") || msg.contains("category"),
        "Expected mention of invalid category, got: {msg}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Full spec integration
// ═══════════════════════════════════════════════════════════════

#[test]
fn parse_full_spec() {
    let input = "\
spec my-feature v2.1.0
title \"My Feature\"

description
  A full-featured spec testing all constructs.
  Second line of description.

motivation
  We need to test everything.
  Multiple motivation lines too.

behavior create-item [happy_path]
  \"Create an item with alias chaining\"

  given
    The system is ready
    @the_user = User { id: \"550e8400\", name: \"Alice\" }

  when create_item
    title = \"Meeting Notes\"
    user_id = @the_user.id

  then returns created item
    assert id is_present
    assert title == \"Meeting Notes\"
    assert created_by == @the_user.id

  then emits process_exit
    assert code == 0

behavior handle-error [error_case]
  \"Handle missing input\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"

depends on user-auth >= 1.0.0
depends on billing >= 2.0.0
";
    let spec = parse(input).unwrap();
    assert_eq!(spec.name, "my-feature");
    assert_eq!(spec.version, "2.1.0");
    assert_eq!(spec.title, "My Feature");
    assert_eq!(spec.behaviors.len(), 2);
    assert_eq!(spec.behaviors[0].preconditions.len(), 2);
    assert_eq!(spec.behaviors[0].action.inputs.len(), 2);
    assert_eq!(spec.behaviors[0].postconditions.len(), 2);
    assert_eq!(spec.dependencies.len(), 2);
}

// ═══════════════════════════════════════════════════════════════
// Trailing content (spec-grammar.spec: reject-trailing-content)
// ═══════════════════════════════════════════════════════════════

/// spec-grammar.spec: reject-trailing-content
// @minter:unit spec-grammar/reject-trailing-content
#[test]
fn trailing_text_after_depends_on() {
    let input = spec_with_deps(
        "\
depends on user-auth >= 1.0.0

this is unexpected trailing content",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = &errors[0].message;
    assert!(
        msg.contains("Unexpected content after end of spec"),
        "Expected trailing content error, got: {msg}"
    );
}

/// spec-grammar.spec: reject-trailing-content
// @minter:unit spec-grammar/reject-trailing-content
#[test]
fn trailing_text_after_last_behavior() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

this is unexpected garbage
";
    let errors = parse(input).unwrap_err();
    assert!(!errors.is_empty());
    // Trailing content after behaviors (no deps) is caught by parse_behaviors
    // as an unrecognized line. Either error message is acceptable — the key
    // invariant is that the file is rejected.
    let msg = &errors[0].message;
    assert!(
        msg.contains("Unexpected content after end of spec") || msg.contains("Expected 'behavior'"),
        "Expected trailing content or bad behavior error, got: {msg}"
    );
}

/// spec-grammar.spec: reject-trailing-content (negative — whitespace is fine)
// @minter:unit spec-grammar/reject-trailing-content
#[test]
fn trailing_whitespace_after_spec_is_ok() {
    let input = "\
spec test-spec v1.0.0
title \"Test\"

description
  A test.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"



";
    let spec = parse(input).unwrap();
    assert_eq!(spec.name, "test-spec");
}

/// spec-grammar.spec: reject-trailing-content
// @minter:unit spec-grammar/reject-trailing-content
#[test]
fn trailing_comments_after_spec() {
    // Non-comment garbage text after the last valid section
    let input = spec_with_deps(
        "\
depends on user-auth >= 1.0.0

random garbage that is not a valid section
more garbage here",
    );
    let errors = parse(&input).unwrap_err();
    assert!(!errors.is_empty());
    let msg = &errors[0].message;
    assert!(
        msg.contains("Unexpected content after end of spec"),
        "Expected trailing content error, got: {msg}"
    );
    // Should include the line number
    assert!(errors[0].line > 0, "Expected a positive line number");
}
