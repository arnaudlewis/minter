spec format-command v1.2.0
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

nfr
  operability#ci-friendly-output


behavior display-spec-grammar [happy_path]
  "Print the functional requirement spec grammar to stdout"

  given
    The format subcommand is invoked with type spec

  when minter format spec

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
    assert output contains "nfr"
    assert output contains "whole-file reference"
    assert output contains "anchor reference"
    assert output contains "override"

  then emits process_exit
    assert code == 0


behavior display-nfr-grammar [happy_path]
  "Print the non-functional requirement spec grammar to stdout"

  given
    The format subcommand is invoked with type nfr

  when minter format nfr

  then emits stdout
    assert output contains "nfr"
    assert output contains "constraint"
    assert output contains "metric"
    assert output contains "threshold"
    assert output contains "rule"
    assert output contains "verification"
    assert output contains "violation"
    assert output contains "overridable"
    assert output contains "environment"
    assert output contains "benchmark"
    assert output contains "pass"
    assert output contains "static"
    assert output contains "runtime"

  then emits process_exit
    assert code == 0


behavior reject-unknown-format-type [error_case]
  "Print error and list valid types when given an unknown format type"

  given
    The format subcommand is invoked with an unrecognized type

  when minter format banana

  then emits stderr
    assert output contains "banana"
    assert output contains "spec"
    assert output contains "nfr"

  then emits process_exit
    assert code == 1


behavior reject-missing-format-type [error_case]
  "Print error and list valid types when no format type is provided"

  given
    The format subcommand is invoked with no arguments

  when minter format

  then emits stderr
    assert output contains "spec"
    assert output contains "nfr"

  then emits process_exit
    assert code == 1


depends on nfr-grammar >= 1.0.0
