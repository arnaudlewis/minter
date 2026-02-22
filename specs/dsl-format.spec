spec dsl-format v1.0.0
title "DSL Format"

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
    A .spec file with a description keyword followed by indented lines

  when parse

  then
    assert description contains all indented lines joined as text


behavior parse-motivation-block [happy_path]
  "Parse a multiline indented motivation block"

  given
    A .spec file with a motivation keyword followed by indented lines

  when parse

  then
    assert motivation contains all indented lines joined as text

# Behavior blocks

behavior parse-behavior-declaration [happy_path]
  "Parse a behavior with name, category, and quoted description"

  given
    A .spec file with: behavior do-thing [happy_path]
    Followed by: "A description of this behavior"

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
    A behavior with: given
    Followed by indented text: The system is ready

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
    assert alias properties contain id and name


behavior parse-given-multiple-preconditions [happy_path]
  "Parse multiple preconditions in a single given block"

  given
    A behavior with given containing both prose and alias declarations

  when parse

  then
    assert all preconditions are captured in order

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
    assert inputs contain name with example "test"
    assert inputs contain count with example 42


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
    assert assertions are captured


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
    assert assertions are captured


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


behavior parse-then-side-effect [happy_path]
  "Parse a side_effect postcondition"

  given
    A behavior with then containing:
    then side_effect
      assert Note entity created with title == "test"

  when parse

  then
    assert postcondition kind == "side_effect"
    assert assertions are captured


behavior parse-multiple-then-blocks [happy_path]
  "Parse multiple postconditions in a single behavior"

  given
    A behavior with multiple then blocks (e.g. emits stdout + emits process_exit)

  when parse

  then
    assert all postconditions are captured in order

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
    A then block containing assertions with no known operator:
    assert assertions are captured
    assert all preconditions are captured in order

  when parse

  then
    assert both assertions are parsed as prose type

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
    A .spec file with multiple depends on declarations

  when parse

  then
    assert all dependencies are captured

# Comments and structure

behavior ignore-comments [happy_path]
  "Lines starting with # are treated as comments and ignored"

  given
    A .spec file with # comment lines between sections and behaviors

  when parse

  then
    assert comments are ignored and parsing succeeds


behavior ignore-blank-lines [happy_path]
  "Blank lines between sections and behaviors are ignored"

  given
    A .spec file with varying numbers of blank lines between elements

  when parse

  then
    assert parsing succeeds regardless of blank line count

# Format errors

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
