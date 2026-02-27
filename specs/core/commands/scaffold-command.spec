spec scaffold-command v1.2.0
title "Scaffold Command"

description
  The minter scaffold command outputs a skeleton .spec file for
  a given type to stdout. For functional requirements it produces
  a complete template with all required sections. For non-functional
  requirements it produces a category-specific template.

motivation
  Starting a new spec from scratch is error-prone. A scaffold command
  gives authors a valid starting point that already satisfies the DSL
  grammar, reducing time to first valid spec.


behavior scaffold-spec [happy_path]
  "Output a functional requirement skeleton with all required sections"

  given
    The scaffold subcommand is invoked with type spec

  when minter scaffold spec

  then emits stdout
    assert output contains "spec"
    assert output contains "title"
    assert output contains "description"
    assert output contains "motivation"
    assert output contains "behavior"
    assert output contains "given"
    assert output contains "when"
    assert output contains "then"
    assert output contains "nfr"

  then emits process_exit
    assert code == 0


behavior scaffold-nfr-with-category [happy_path]
  "Output a non-functional requirement skeleton for a specific category"

  given
    The scaffold subcommand is invoked with type nfr and category performance

  when minter scaffold nfr performance

  then emits stdout
    assert output contains "nfr"
    assert output contains "constraint"
    assert output contains "verification"
    assert output contains "violation"
    assert output contains "overridable"
    assert output contains "performance"

  then emits process_exit
    assert code == 0


behavior scaffold-nfr-all-categories [happy_path]
  "Accept all seven valid NFR categories without error"

  given
    The scaffold subcommand is invoked with type nfr and category security

  when minter scaffold nfr security

  then emits stdout
    assert output contains "security"

  then emits process_exit
    assert code == 0


behavior reject-unknown-nfr-category [error_case]
  "Print error and list valid categories when given an unknown NFR category"

  given
    The scaffold subcommand is invoked with type nfr and an unrecognized category

  when minter scaffold nfr banana

  then emits stderr
    assert output contains "banana"
    assert output contains "performance"
    assert output contains "reliability"
    assert output contains "security"
    assert output contains "observability"
    assert output contains "scalability"
    assert output contains "cost"
    assert output contains "operability"

  then emits process_exit
    assert code == 1


behavior reject-nfr-missing-category [error_case]
  "Print error when nfr type is used without a category argument"

  given
    The scaffold subcommand is invoked with type nfr but no category

  when minter scaffold nfr

  then emits stderr
    assert output contains "category"

  then emits process_exit
    assert code == 1


behavior reject-unknown-scaffold-type [error_case]
  "Print error and list valid types when given an unknown scaffold type"

  given
    The scaffold subcommand is invoked with an unrecognized type

  when minter scaffold banana

  then emits stderr
    assert output contains "banana"
    assert output contains "spec"
    assert output contains "nfr"

  then emits process_exit
    assert code == 1


behavior scaffold-output-is-parseable [edge_case]
  "The generated scaffold passes validation when saved to a file"

  given
    The scaffold spec output is written to a file named scaffolded.spec

  when minter validate scaffolded.spec

  then emits process_exit
    assert code == 0


behavior scaffold-nfr-output-is-parseable [edge_case]
  "The generated NFR scaffold passes validation when saved to a file"

  given
    The scaffold nfr performance output is written to a file named scaffolded.nfr

  when minter validate scaffolded.nfr

  then emits process_exit
    assert code == 0


depends on nfr-grammar >= 1.0.0
