mod common;

use common::{minter, temp_spec, VALID_SPEC};
use predicates::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Header parsing — happy paths (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-spec-declaration
#[test]
fn parse_spec_declaration() {
    let (_dir, path) = temp_spec("my-feature", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-title
#[test]
fn parse_title() {
    let spec = "\
spec test-spec v1.0.0
title \"A Descriptive Title With Spaces\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("title-test", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-description-block
#[test]
fn parse_description_block() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  This is a multiline description block.
  It spans several lines of text.
  Each line is indented.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("desc-test", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-motivation-block
#[test]
fn parse_motivation_block() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  This is a multiline motivation block.
  Explaining why this feature exists.
  Over multiple indented lines.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("motiv-test", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Behavior blocks — happy paths (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-behavior-declaration
#[test]
fn parse_behavior_declaration() {
    let (_dir, path) = temp_spec("behavior-test", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-category-happy-path
#[test]
fn parse_category_happy_path() {
    let (_dir, path) = temp_spec("cat-happy", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-category-error-case
#[test]
fn parse_category_error_case() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Happy path\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

behavior fail-thing [error_case]
  \"Error case\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
";
    let (_dir, path) = temp_spec("cat-error", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-category-edge-case
#[test]
fn parse_category_edge_case() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Happy path\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

behavior weird-thing [edge_case]
  \"Edge case\"

  given
    Unusual conditions

  when act

  then emits stdout
    assert output contains \"handled\"
";
    let (_dir, path) = temp_spec("cat-edge", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Given section — happy paths (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-given-prose
#[test]
fn parse_given_prose() {
    let (_dir, path) = temp_spec("given-prose", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-given-alias-declaration
#[test]
fn parse_given_alias_declaration() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-alias [happy_path]
  \"Has alias declaration\"

  given
    @the_user = User { id: \"550e8400\", name: \"Alice\" }

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("alias-decl", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-given-multiple-preconditions
#[test]
fn parse_given_multiple_preconditions() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-multi-given [happy_path]
  \"Has prose and alias preconditions\"

  given
    The system is ready
    @the_user = User { id: \"550e8400\", name: \"Alice\" }

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("multi-given", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// When section — happy paths (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-when-action
#[test]
fn parse_when_action() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-action [happy_path]
  \"Has a named action\"

  given
    Ready

  when create_item

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("when-action", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-when-inputs
#[test]
fn parse_when_inputs() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-inputs [happy_path]
  \"Has typed inputs with examples\"

  given
    Ready

  when create_item
    name = \"test\"
    count = 42

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("when-inputs", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-when-alias-reference
#[test]
fn parse_when_alias_reference() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-alias-ref [happy_path]
  \"References alias in when\"

  given
    @the_user = User { id: \"550e8400\" }

  when act
    user_id = @the_user.id

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("when-alias-ref", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Then section — postcondition kinds (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-then-returns
#[test]
fn parse_then_returns() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-returns [happy_path]
  \"Has returns postcondition\"

  given
    Ready

  when act

  then returns created item
    assert id is_present
    assert name == \"test\"
";
    let (_dir, path) = temp_spec("then-returns", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-then-emits
#[test]
fn parse_then_emits() {
    let (_dir, path) = temp_spec("then-emits", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-then-emits-process-exit
#[test]
fn parse_then_emits_process_exit() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-exit [happy_path]
  \"Has process_exit postcondition\"

  given
    Ready

  when act

  then emits process_exit
    assert code == 0
";
    let (_dir, path) = temp_spec("then-exit", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-then-side-effect
#[test]
fn parse_then_side_effect() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-side-effect [happy_path]
  \"Has side_effect postcondition\"

  given
    Ready

  when act

  then side_effect
    assert Note entity created with title == \"test\"
";
    let (_dir, path) = temp_spec("then-side-effect", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-multiple-then-blocks
#[test]
fn parse_multiple_then_blocks() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-multi-then [happy_path]
  \"Has multiple then blocks\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

  then emits process_exit
    assert code == 0
";
    let (_dir, path) = temp_spec("multi-then", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Assertions — happy paths (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-assert-equals
#[test]
fn parse_assert_equals() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-equals [happy_path]
  \"Has equals assertion\"

  given
    Ready

  when act

  then returns result
    assert name == \"test\"
";
    let (_dir, path) = temp_spec("assert-equals", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-assert-is-present
#[test]
fn parse_assert_is_present() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-is-present [happy_path]
  \"Has is_present assertion\"

  given
    Ready

  when act

  then returns result
    assert id is_present
";
    let (_dir, path) = temp_spec("assert-is-present", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-assert-contains
#[test]
fn parse_assert_contains() {
    let (_dir, path) = temp_spec("assert-contains", VALID_SPEC);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-assert-in-range
#[test]
fn parse_assert_in_range() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-range [happy_path]
  \"Has in_range assertion\"

  given
    Ready

  when act

  then returns result
    assert count in_range 1..100
";
    let (_dir, path) = temp_spec("assert-range", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-assert-matches-pattern
#[test]
fn parse_assert_matches_pattern() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-pattern [happy_path]
  \"Has matches_pattern assertion\"

  given
    Ready

  when act

  then returns result
    assert email matches_pattern \"^.+@.+$\"
";
    let (_dir, path) = temp_spec("assert-pattern", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-assert-equals-ref
#[test]
fn parse_assert_equals_ref() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-ref-assert [happy_path]
  \"Has equals_ref assertion\"

  given
    @the_user = User { id: \"123\" }

  when act

  then returns result
    assert created_by == @the_user.id
";
    let (_dir, path) = temp_spec("assert-ref", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-assert-greater-or-equal
#[test]
fn parse_assert_greater_or_equal() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior with-gte [happy_path]
  \"Has >= assertion\"

  given
    Ready

  when act

  then returns result
    assert count >= 2
";
    let (_dir, path) = temp_spec("assert-gte", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Dependencies — happy paths (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-depends-on
#[test]
fn parse_depends_on() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

depends on user-auth >= 1.0.0
";
    let (_dir, path) = temp_spec("with-dep", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: parse-multiple-dependencies
#[test]
fn parse_multiple_dependencies() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

depends on user-auth >= 1.0.0
depends on billing >= 2.0.0
";
    let (_dir, path) = temp_spec("multi-deps", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Assertions — prose (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: parse-assert-prose
#[test]
fn parse_assert_prose() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Prose assertions\"

  given
    Ready

  when act

  then
    assert assertions are captured
    assert all preconditions are captured in order
";
    let (_dir, path) = temp_spec("prose-assert", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Comments and structure — happy paths (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: ignore-comments
#[test]
fn ignore_comments() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

# This is a comment between header and behaviors

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

# Another comment between behaviors

behavior another-thing [happy_path]
  \"Another\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("comments", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

/// dsl-format.spec: ignore-blank-lines
#[test]
fn ignore_blank_lines() {
    let spec = "\
spec test-spec v1.0.0

title \"Test\"


description
  Test.


motivation
  Test.


behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

";
    let (_dir, path) = temp_spec("blank-lines", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

// ═══════════════════════════════════════════════════════════════
// Format errors — error cases (dsl-format.spec)
// ═══════════════════════════════════════════════════════════════

/// dsl-format.spec: reject-behavior-without-given
#[test]
fn reject_behavior_without_given() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior bad [happy_path]
  \"No given section\"

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("no-given", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("given").or(predicate::str::contains("Given")));
}

/// dsl-format.spec: reject-behavior-without-when
#[test]
fn reject_behavior_without_when() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior bad [happy_path]
  \"No when section\"

  given
    Ready

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("no-when", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("when").or(predicate::str::contains("When")));
}

/// dsl-format.spec: reject-behavior-without-then
#[test]
fn reject_behavior_without_then() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior bad [happy_path]
  \"No then section\"

  given
    Ready

  when act
";
    let (_dir, path) = temp_spec("no-then", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("then").or(predicate::str::contains("Then")));
}

/// dsl-format.spec: reject-wrong-section-order
#[test]
fn reject_wrong_section_order() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior bad [happy_path]
  \"Wrong order\"

  given
    Ready

  then emits stdout
    assert output contains \"done\"

  when act
";
    let (_dir, path) = temp_spec("wrong-order", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// dsl-format.spec: reject-assert-without-field
#[test]
fn reject_assert_without_field() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior bad [happy_path]
  \"Malformed assertion\"

  given
    Ready

  when act

  then emits stdout
    assert == \"test\"
";
    let (_dir, path) = temp_spec("no-field", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// dsl-format.spec: reject-unknown-assertion-operator
#[test]
fn reject_unknown_assertion_operator() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior bad [happy_path]
  \"Unknown operator\"

  given
    Ready

  when act

  then emits stdout
    assert name frobnicates \"test\"
";
    let (_dir, path) = temp_spec("bad-operator", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("frobnicates").or(predicate::str::contains("operator")));
}

/// dsl-format.spec: reject-alias-without-entity
#[test]
fn reject_alias_without_entity() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior bad [happy_path]
  \"Alias with no entity type\"

  given
    @my_alias = { id: \"123\" }

  when act

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("no-entity", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// dsl-format.spec: reject-malformed-alias-reference
#[test]
fn reject_malformed_alias_reference() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior bad [happy_path]
  \"Malformed alias reference\"

  given
    The system is ready

  when act
    user_id = @

  then emits stdout
    assert output contains \"done\"
";
    let (_dir, path) = temp_spec("bad-ref", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// dsl-format.spec: reject-depends-on-without-version
#[test]
fn reject_depends_on_without_version() {
    let spec = "\
spec test-spec v1.0.0
title \"Test\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Do it\"

  given
    Ready

  when act

  then emits stdout
    assert output contains \"done\"

depends on user-auth
";
    let (_dir, path) = temp_spec("dep-no-version", spec);
    minter()
        .arg("validate")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("version").or(predicate::str::contains("user-auth")));
}
