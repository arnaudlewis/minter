spec spec-grammar v1.2.0
title "Spec Grammar"

description
  Defines the grammar and structure of the .spec DSL. Every keyword,
  section, block structure, operator, and syntax rule that the parser
  must recognize and enforce.

motivation
  The DSL is the authoring interface for specs. A strict, well-defined
  format ensures consistency across all specs and enables reliable
  parsing. This spec is the contract for what constitutes a well-formed
  .spec file.

# Header section

behavior parse-spec-declaration [happy_path]
  "Parse the spec declaration with name and version"

  given
    A .spec file starting with: spec my-feature v1.0.0

  when parse

  then
    assert name == "my-feature"
    assert version == "1.0.0"


behavior parse-title [happy_path]
  "Parse the title as a quoted string"

  given
    A .spec file with: title "My Feature"

  when parse

  then
    assert title == "My Feature"


behavior parse-description-block [happy_path]
  "Parse a multiline indented description block"

  given
    A .spec file with:
    description
      This is line one.
      This is line two.

  when parse

  then
    assert description == "This is line one. This is line two."


behavior parse-motivation-block [happy_path]
  "Parse a multiline indented motivation block"

  given
    A .spec file with:
    motivation
      Reason line one.
      Reason line two.

  when parse

  then
    assert motivation == "Reason line one. Reason line two."


behavior parse-empty-description [edge_case]
  "Parse a description block with an empty string"

  given
    A .spec file with:
    description
      (a single blank indented line)

  when parse

  then
    assert description == ""


# Behavior blocks

behavior parse-behavior-declaration [happy_path]
  "Parse a behavior with name, category, and quoted description"

  given
    A .spec file with:
    behavior do-thing [happy_path]
      "A description of this behavior"

  when parse

  then
    assert behavior name == "do-thing"
    assert behavior category == "happy_path"
    assert behavior description == "A description of this behavior"


behavior parse-category-happy-path [happy_path]
  "Accept happy_path as a valid category"

  given
    A behavior with category [happy_path]

  when parse

  then
    assert category == "happy_path"


behavior parse-category-error-case [happy_path]
  "Accept error_case as a valid category"

  given
    A behavior with category [error_case]

  when parse

  then
    assert category == "error_case"


behavior parse-category-edge-case [happy_path]
  "Accept edge_case as a valid category"

  given
    A behavior with category [edge_case]

  when parse

  then
    assert category == "edge_case"


# Given section

behavior parse-given-prose [happy_path]
  "Parse a prose precondition in the given section"

  given
    A behavior with:
    given
      The system is ready

  when parse

  then
    assert precondition description == "The system is ready"


behavior parse-given-alias-declaration [happy_path]
  "Parse an alias declaration with entity and properties"

  given
    A behavior with given containing:
    @the_user = User { id: "550e8400", name: "Alice" }

  when parse

  then
    assert alias name == "the_user"
    assert alias entity == "User"
    assert alias property id == "550e8400"
    assert alias property name == "Alice"


behavior parse-given-alias-single-property [edge_case]
  "Parse an alias with exactly one property"

  given
    A behavior with given containing:
    @token = Token { value: "abc123" }

  when parse

  then
    assert alias name == "token"
    assert alias entity == "Token"
    assert alias property value == "abc123"


behavior parse-given-alias-zero-properties [edge_case]
  "Parse an alias with no properties"

  given
    A behavior with given containing:
    @empty = EmptyEntity {}

  when parse

  then
    assert alias name == "empty"
    assert alias entity == "EmptyEntity"


behavior parse-given-multiple-preconditions [happy_path]
  "Parse multiple preconditions in a single given block"

  given
    A behavior with given containing:
    The database is seeded
    @the_user = User { id: "1", name: "Alice" }
    The user is logged in

  when parse

  then
    assert preconditions count == 3
    assert precondition 1 is prose "The database is seeded"
    assert precondition 2 is alias "the_user"
    assert precondition 3 is prose "The user is logged in"


# When section

behavior parse-when-action [happy_path]
  "Parse the action name from the when section"

  given
    A behavior with: when create_item

  when parse

  then
    assert action == "create_item"


behavior parse-when-inputs [happy_path]
  "Parse named inputs with example values"

  given
    A behavior with when containing:
    name = "test"
    count = 42

  when parse

  then
    assert input name == "test"
    assert input count == 42


behavior parse-when-alias-reference [happy_path]
  "Parse an input that references an alias from given"

  given
    A behavior with when containing:
    user_id = @the_user.id

  when parse

  then
    assert input user_id references alias "the_user" field "id"


# Then section — postcondition kinds

behavior parse-then-returns [happy_path]
  "Parse a returns postcondition with assertions"

  given
    A behavior with then containing:
    then returns created item
      assert id is_present
      assert name == "test"

  when parse

  then
    assert postcondition kind == "returns"
    assert postcondition description == "created item"
    assert assertion count == 2


behavior parse-then-emits [happy_path]
  "Parse an emits postcondition with assertions"

  given
    A behavior with then containing:
    then emits stdout
      assert output contains "done"

  when parse

  then
    assert postcondition kind == "emits"
    assert postcondition target == "stdout"
    assert assertion count == 1


behavior parse-then-emits-process-exit [happy_path]
  "Parse a process_exit postcondition"

  given
    A behavior with then containing:
    then emits process_exit
      assert code == 0

  when parse

  then
    assert postcondition kind == "emits"
    assert postcondition target == "process_exit"
    assert assertion count == 1


behavior parse-then-side-effect [happy_path]
  "Parse a side_effect postcondition"

  given
    A behavior with then containing:
    then side_effect
      assert Note entity created with title == "test"

  when parse

  then
    assert postcondition kind == "side_effect"
    assert assertion count == 1


behavior parse-then-plain [happy_path]
  "Parse a then block with no kind qualifier"

  given
    A behavior with then containing:
    then
      assert name == "test"
      assert count >= 1

  when parse

  then
    assert postcondition kind == "plain"
    assert assertion count == 2


behavior parse-multiple-then-blocks [happy_path]
  "Parse multiple postconditions in a single behavior"

  given
    A behavior with:
    then emits stdout
      assert output contains "done"
    then emits process_exit
      assert code == 0

  when parse

  then
    assert postcondition count == 2
    assert postcondition 1 kind == "emits"
    assert postcondition 1 target == "stdout"
    assert postcondition 2 kind == "emits"
    assert postcondition 2 target == "process_exit"


# Assertions

behavior parse-assert-equals [happy_path]
  "Parse an equality assertion"

  given
    An assertion: assert name == "test"

  when parse

  then
    assert kind == "equals"
    assert field == "name"
    assert value == "test"


behavior parse-assert-is-present [happy_path]
  "Parse a presence assertion"

  given
    An assertion: assert id is_present

  when parse

  then
    assert kind == "is_present"
    assert field == "id"


behavior parse-assert-contains [happy_path]
  "Parse a contains assertion"

  given
    An assertion: assert output contains "done"

  when parse

  then
    assert kind == "contains"
    assert field == "output"
    assert value == "done"


behavior parse-assert-in-range [happy_path]
  "Parse a range assertion"

  given
    An assertion: assert count in_range 1..100

  when parse

  then
    assert kind == "in_range"
    assert field == "count"
    assert min == 1
    assert max == 100


behavior parse-assert-matches-pattern [happy_path]
  "Parse a pattern matching assertion"

  given
    An assertion: assert email matches_pattern "^.+@.+$"

  when parse

  then
    assert kind == "matches_pattern"
    assert field == "email"
    assert pattern == "^.+@.+$"


behavior parse-assert-equals-ref [happy_path]
  "Parse an assertion that references an alias"

  given
    An assertion: assert created_by == @the_user.id

  when parse

  then
    assert kind == "equals_ref"
    assert field == "created_by"
    assert reference alias == "the_user"
    assert reference field == "id"


behavior parse-assert-greater-or-equal [happy_path]
  "Parse a comparison assertion"

  given
    An assertion: assert count >= 2

  when parse

  then
    assert kind == "greater_or_equal"
    assert field == "count"
    assert value == 2


behavior parse-assert-prose [happy_path]
  "Parse a prose assertion with no known operator"

  given
    A then block containing:
    assert all preconditions are captured in order

  when parse

  then
    assert kind == "prose"
    assert text == "all preconditions are captured in order"


# Unicode and special characters

behavior parse-unicode-in-quoted-strings [edge_case]
  "Parse quoted strings containing unicode characters"

  given
    A .spec file with: title "Spécification — Feature"

  when parse

  then
    assert title == "Spécification — Feature"


behavior parse-trailing-whitespace [edge_case]
  "Ignore trailing whitespace on indented lines"

  given
    A .spec file where indented description lines end with trailing spaces

  when parse

  then
    assert description does not contain trailing spaces


# Indentation rules

behavior accept-two-space-indentation [happy_path]
  "Accept indentation of two or more spaces for indented blocks"

  given
    A .spec file with:
    description
      Two-space indented content.

  when parse

  then
    assert description == "Two-space indented content."


behavior reject-tab-indentation [error_case]
  "Reject tabs as indentation characters"

  given
    A .spec file where a description line is indented with a tab character

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions invalid indentation

  then emits process_exit
    assert code == 1


behavior reject-zero-indent-in-block [error_case]
  "Reject lines with no indentation inside an indented block"

  given
    A .spec file where a description block contains a line with zero indentation
    that is not a keyword or blank line

  when parse

  then emits stderr
    assert output contains line number

  then emits process_exit
    assert code == 1


# Dependencies

behavior parse-depends-on [happy_path]
  "Parse a dependency declaration with name and version constraint"

  given
    A .spec file with: depends on user-auth >= 1.0.0

  when parse

  then
    assert dependency spec == "user-auth"
    assert dependency version constraint == ">= 1.0.0"


behavior parse-multiple-dependencies [happy_path]
  "Parse multiple depends on lines"

  given
    A .spec file with:
    depends on user-auth >= 1.0.0
    depends on billing >= 2.0.0

  when parse

  then
    assert dependency count == 2
    assert dependency 1 spec == "user-auth"
    assert dependency 2 spec == "billing"


# Comments and structure

behavior ignore-comments [happy_path]
  "Lines starting with # are treated as comments and ignored"

  given
    A .spec file with:
    spec my-feature v1.0.0
    # This is a comment
    title "My Feature"

  when parse

  then
    assert name == "my-feature"
    assert title == "My Feature"


behavior ignore-blank-lines [happy_path]
  "Blank lines between sections and behaviors are ignored"

  given
    A .spec file with three blank lines between the title and description

  when parse

  then
    assert title is_present
    assert description is_present


# Format errors — structural

behavior reject-missing-spec-declaration [error_case]
  "Reject a file that does not start with a spec declaration"

  given
    A .spec file that begins with: title "My Feature"
    without a preceding spec declaration line

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing spec declaration

  then emits process_exit
    assert code == 1


behavior reject-missing-title [error_case]
  "Reject a spec that has no title line"

  given
    A .spec file with a spec declaration and description but no title line

  when parse

  then emits stderr
    assert output mentions missing title

  then emits process_exit
    assert code == 1


behavior reject-missing-description [error_case]
  "Reject a spec that has no description block"

  given
    A .spec file with a spec declaration and title but no description block

  when parse

  then emits stderr
    assert output mentions missing description

  then emits process_exit
    assert code == 1


behavior reject-missing-motivation [error_case]
  "Reject a spec that has no motivation block"

  given
    A .spec file with a spec declaration, title, and description but no
    motivation block

  when parse

  then emits stderr
    assert output mentions missing motivation

  then emits process_exit
    assert code == 1


behavior reject-behavior-without-description [error_case]
  "Reject a behavior that has no quoted description string"

  given
    A .spec file with:
    behavior do-thing [happy_path]
    given
      Some condition

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing behavior description

  then emits process_exit
    assert code == 1


# Format errors — behavior sections

behavior reject-behavior-without-given [error_case]
  "Reject a behavior that has when and then but no given"

  given
    A behavior block with no given section

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing given

  then emits process_exit
    assert code == 1


behavior reject-behavior-without-when [error_case]
  "Reject a behavior that has given and then but no when"

  given
    A behavior block with no when section

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing when

  then emits process_exit
    assert code == 1


behavior reject-behavior-without-then [error_case]
  "Reject a behavior that has given and when but no then"

  given
    A behavior block with no then section

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing then

  then emits process_exit
    assert code == 1


behavior reject-wrong-section-order [error_case]
  "Reject a behavior where given/when/then are out of order"

  given
    A behavior block where then appears before when

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions section ordering

  then emits process_exit
    assert code == 1


# Format errors — assertions and aliases

behavior reject-assert-without-field [error_case]
  "Reject an assertion with no field name"

  given
    A then block containing: assert == "test"

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions malformed assertion

  then emits process_exit
    assert code == 1


behavior reject-unknown-assertion-operator [error_case]
  "Reject an assertion with an unrecognized operator"

  given
    A then block containing: assert name frobnicates "test"

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions unrecognized operator

  then emits process_exit
    assert code == 1


behavior reject-alias-without-entity [error_case]
  "Reject an alias declaration with no entity type"

  given
    A given block containing: @my_alias = { id: "123" }

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing entity type

  then emits process_exit
    assert code == 1


behavior reject-malformed-alias-reference [error_case]
  "Reject an alias reference with invalid syntax"

  given
    A when block containing: user_id = @

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions malformed alias reference

  then emits process_exit
    assert code == 1


# NFR reference sections

behavior parse-spec-level-nfr-section [happy_path]
  "Parse whole-file and anchor references in the spec-level nfr section"

  given
    A .spec file with:
    nfr
      performance
      security#tls-required

  when parse

  then
    assert nfr ref count == 2
    assert nfr ref 1 category == "performance"
    assert nfr ref 1 kind == "whole-file"
    assert nfr ref 2 category == "security"
    assert nfr ref 2 anchor == "tls-required"


behavior parse-behavior-level-nfr-section [happy_path]
  "Parse anchor and override references in a behavior-level nfr section"

  given
    A behavior with:
    nfr
      performance#response-time
      performance#response-time < 200ms

  when parse

  then
    assert nfr ref count == 2
    assert nfr ref 1 kind == "anchor"
    assert nfr ref 2 kind == "override"
    assert nfr ref 2 operator == "<"
    assert nfr ref 2 value == "200ms"


# Format errors — dependencies

behavior reject-depends-on-without-version [error_case]
  "Reject a dependency declaration with no version constraint"

  given
    A .spec file with: depends on user-auth

  when parse

  then emits stderr
    assert output contains line number
    assert output mentions missing version constraint

  then emits process_exit
    assert code == 1
