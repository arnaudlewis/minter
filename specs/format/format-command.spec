spec format-command v1.0.0
title "Format Command"

description
  The minter format command displays the grammar reference for
  a given spec type. Outputs the DSL structure to stdout so authors
  can see the expected format without opening documentation.

motivation
  Spec authors need a quick reference for the DSL grammar while
  writing specs. A built-in format command eliminates context
  switching to external docs and ensures the reference always
  matches the parser's actual grammar.


behavior display-fr-grammar [happy_path]
  "Print the functional requirement spec grammar to stdout"

  given
    The format subcommand is invoked with type fr

  when minter format fr

  then emits stdout
    assert output contains "spec"
    assert output contains "title"
    assert output contains "description"
    assert output contains "motivation"
    assert output contains "behavior"
    assert output contains "given"
    assert output contains "when"
    assert output contains "then"
    assert output contains "assert"
    assert output contains "depends on"

  then emits process_exit
    assert code == 0


behavior display-nfr-grammar [happy_path]
  "Print the non-functional requirement spec grammar to stdout"

  given
    The format subcommand is invoked with type nfr

  when minter format nfr

  then emits stdout
    assert output contains "constraint"
    assert output contains "verification"
    assert output contains "references"
    assert output contains "overrides"

  then emits process_exit
    assert code == 0


behavior reject-unknown-format-type [error_case]
  "Print error and list valid types when given an unknown format type"

  given
    The format subcommand is invoked with an unrecognized type

  when minter format banana

  then emits stderr
    assert output contains "banana"
    assert output contains "fr"
    assert output contains "nfr"

  then emits process_exit
    assert code == 1


behavior reject-missing-format-type [error_case]
  "Print error and list valid types when no format type is provided"

  given
    The format subcommand is invoked with no arguments

  when minter format

  then emits stderr
    assert output contains "fr"
    assert output contains "nfr"

  then emits process_exit
    assert code == 1
