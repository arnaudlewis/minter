/// Display the DSL grammar reference for FR or NFR spec types.
pub fn run_format(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("missing format type. Valid types: fr, nfr");
        return 1;
    }
    match args[0].as_str() {
        "fr" => {
            print_fr_grammar();
            0
        }
        "nfr" => {
            print_nfr_grammar();
            0
        }
        other => {
            eprintln!("unknown format type: {other}. Valid types: fr, nfr");
            1
        }
    }
}

fn print_fr_grammar() {
    println!(
        "\
Functional Requirement (FR) Spec Grammar
=========================================

spec <name> v<version>
title \"<title>\"

description
  <free text>

motivation
  <free text>

behavior <name> [<category>]
  \"<description>\"

  given
    <precondition text>
    @<alias> = <Entity> {{ <key>: <value>, ... }}

  when <action-name>
    <input> = <value>
    <input> = @<alias>.<field>

  then returns <channel>
    assert <field> == <value>
    assert <field> contains <value>
    assert <field> is_present
    assert <field> in_range <min>..<max>
    assert <field> matches_pattern <pattern>
    assert <field> >= <value>

  then emits <channel>
    assert <field> == <value>

  then side_effect
    assert <prose description>

depends on <spec-name> >= <version>

Categories: happy_path, error_case, edge_case"
    );
}

fn print_nfr_grammar() {
    println!(
        "\
Non-Functional Requirement (NFR) Spec Grammar
===============================================

spec <name> v<version>
title \"<title>\"

description
  <free text>

motivation
  <free text>

behavior <name> [<category>]
  \"<description>\"

  given
    <precondition text>

  when <action-name>

  then returns <channel>
    assert <field> == <value>

NFR-Specific Keywords:
  constraint    — defines a non-functional constraint
  verification  — describes how the constraint is verified
  references    — links to external standards or documents
  overrides     — indicates this NFR overrides another spec

depends on <spec-name> >= <version>"
    );
}
