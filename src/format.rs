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

nfr <category> v<version>
title \"<title>\"

description
  <free text>

motivation
  <free text>


constraint <name> [metric]
  \"<description>\"

  metric \"<what is measured>\"
  threshold <operator> <value>

  verification
    environment <env1>, <env2>, ...
    benchmark \"<procedure>\"
    dataset \"<data requirements>\"
    pass \"<criteria>\"

  violation <critical|high|medium|low>
  overridable <yes|no>


constraint <name> [rule]
  \"<description>\"

  rule
    <free text>

  verification
    static \"<check description>\"
    runtime \"<check description>\"

  violation <critical|high|medium|low>
  overridable <yes|no>

Categories: performance, reliability, security, observability, scalability, cost, operability
Threshold operators: <, >, <=, >=, =="
    );
}
