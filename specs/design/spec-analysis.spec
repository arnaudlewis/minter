spec spec-analysis v1.0.0
title "Spec Analysis"

description
  Reads .spec files from a project directory and produces an analysis
  result that drives design system generation. Extracts entities from
  spec definitions, classifies the project into a structural archetype
  (dashboard, e-commerce, content, form-heavy, or generic), and collects
  spec metadata (names grouped by directory into domains, per-spec
  metrics, project name). The archetype detection uses structural signals
  — spec count, hierarchy depth, dependency patterns, and web-command
  presence — not just entity keyword matching. The analysis result
  includes a reasoning string explaining why the archetype was chosen
  and a qualitative app description derived from entity and structural
  analysis.

motivation
  Design generation needs to understand what kind of application the
  specs describe before choosing palettes, layouts, and typography.
  Keyword-only archetype detection fails for projects like minter where
  domain entities (spec, behavior, nfr) do not match dashboard keywords
  (metrics, analytics). Structural analysis — high spec count, deep
  hierarchy, web-command dependency — provides reliable classification
  even when entity keywords are domain-specific. Spec metadata feeds
  real content into the design preview so the prototype shows actual
  spec names, behavior counts, and domain groupings instead of
  placeholder text.

nfr
  operability#deterministic-output
  reliability#no-silent-data-loss


# Entity extraction

behavior extract-entities-from-specs [happy_path]
  "Collect unique entity names referenced across all spec files"

  given
    A directory containing specs that reference entities
    named user, repo, spec, and behavior

  when analyze specs at the directory path

  then returns analysis_result
    assert entities contains "user"
    assert entities contains "repo"
    assert entities contains "spec"
    assert entities contains "behavior"
    assert entity count == 4


behavior extract-entities-deduplicates [edge_case]
  "The same entity referenced in multiple specs appears only once"

  given
    specs/a.spec references entities named user and repo
    specs/b.spec references entities named user and config

  when analyze specs at the directory path

  then returns analysis_result
    assert entities contains "user" exactly once
    assert entity count == 3


# Archetype classification — structural signals

behavior classify-dashboard-by-structure [happy_path]
  "Detect dashboard archetype from structural signals when entity keywords score low"

  given
    A directory containing 20+ specs
    The dependency hierarchy has depth 5 or greater
    A spec named web-command exists in the tree
    Entity keywords do not match any archetype (e.g., spec, behavior, nfr)

  when analyze specs at the directory path

  then returns analysis_result
    assert archetype name == "dashboard"
    assert archetype confidence >= 0.7
    assert archetype reasoning contains "spec count"
    assert archetype reasoning contains "hierarchy" or "depth"
    assert archetype reasoning contains "web"


behavior classify-ecommerce-by-entities [happy_path]
  "Detect e-commerce archetype from product and cart entity keywords"

  given
    A directory containing specs that reference entities
    named product, cart, order, and payment

  when analyze specs at the directory path

  then returns analysis_result
    assert archetype name == "e-commerce"
    assert archetype confidence >= 0.5


behavior classify-content-by-entities [happy_path]
  "Detect content archetype from article and content entity keywords"

  given
    A directory containing specs that reference entities
    named article, post, author, and comment

  when analyze specs at the directory path

  then returns analysis_result
    assert archetype name == "content"
    assert archetype confidence >= 0.5


behavior classify-form-heavy-by-entities [happy_path]
  "Detect form-heavy archetype from form and wizard entity keywords"

  given
    A directory containing specs that reference entities
    named form, wizard, step, and validation

  when analyze specs at the directory path

  then returns analysis_result
    assert archetype name == "form-heavy"
    assert archetype confidence >= 0.5


behavior classify-generic-fallback [happy_path]
  "Fall back to generic archetype when no structural or entity signals match"

  given
    A directory containing 3 specs with no matching entity keywords
    The dependency hierarchy has depth 1
    No web-command spec exists

  when analyze specs at the directory path

  then returns analysis_result
    assert archetype name == "generic"
    assert archetype confidence < 0.5
    assert archetype reasoning contains "no strong" or "insufficient"


behavior structural-overrides-keyword [edge_case]
  "Structural signals override low entity keyword scores"

  given
    A directory containing 25 specs with domain-specific entities
    Entity keyword matching produces confidence 0.14
    The dependency hierarchy has depth 8
    A web-command spec exists

  when analyze specs at the directory path

  then returns analysis_result
    assert archetype name != "generic"
    assert archetype confidence > 0.14


# Archetype reasoning

behavior archetype-reasoning-explains-why [happy_path]
  "The reasoning string explains which signals led to the classification"

  given
    A directory containing specs that classify as dashboard

  when analyze specs at the directory path

  then returns analysis_result
    assert archetype reasoning is not empty
    assert archetype reasoning length > 20


# Spec metadata extraction

behavior extract-project-name [happy_path]
  "Derive project name from the root directory name"

  given
    The spec directory is located at /projects/minter/specs/

  when analyze specs at the directory path

  then returns analysis_result
    assert project name == "minter" or "Minter"


behavior extract-domains-from-directories [happy_path]
  "Group specs by their parent directory into named domains"

  given
    A directory structure:
    specs/core/commands/validate-command.spec
    specs/core/commands/watch-command.spec
    specs/grammar/spec-grammar.spec
    specs/mcp/mcp-server.spec

  when analyze specs at the directory path

  then returns analysis_result
    assert domains contains a domain with specs including "validate-command" and "watch-command"
    assert domains contains a domain with specs including "spec-grammar"
    assert domains contains a domain with specs including "mcp-server"
    assert each domain has a name
    assert each domain has a spec count


behavior extract-per-spec-metrics [happy_path]
  "Collect name, version, and behavior count for each spec"

  given
    specs/auth.spec at version 2.1.0 has 15 behaviors
    specs/config.spec at version 1.0.0 has 8 behaviors

  when analyze specs at the directory path

  then returns analysis_result
    assert spec metrics contains an entry for "auth" with version "2.1.0" and behavior count 15
    assert spec metrics contains an entry for "config" with version "1.0.0" and behavior count 8


behavior extract-aggregate-metrics [happy_path]
  "Compute total spec count, total behavior count, and total entity count"

  given
    A directory with 5 specs containing 42 behaviors and 7 unique entities

  when analyze specs at the directory path

  then returns analysis_result
    assert total spec count == 5
    assert total behavior count == 42
    assert total entity count == 7


# App description

behavior generate-app-description [happy_path]
  "Produce a qualitative description of the application from analysis"

  given
    A directory containing specs for a developer tool with validation,
    coverage tracking, and graph visualization

  when analyze specs at the directory path

  then returns analysis_result
    assert app description is not empty
    assert app description length > 10


# Error cases

behavior handle-empty-directory [error_case]
  "Return an error when no spec files are found"

  given
    An empty directory with no .spec files

  when analyze specs at the directory path

  then returns error
    assert error message contains "no spec" or "no .spec"


behavior handle-nonexistent-directory [error_case]
  "Return an error when the directory does not exist"

  given
    The path /nonexistent/path/ does not exist on disk

  when analyze specs at the directory path

  then returns error
    assert error message contains "not found" or "does not exist"


behavior handle-all-unparseable-specs [error_case]
  "Return an error when every spec file fails to parse"

  given
    A directory where all .spec files contain invalid syntax

  when analyze specs at the directory path

  then returns error
    assert error message contains "parse" or "invalid"


behavior handle-partial-parse-failure [edge_case]
  "Analyze successfully parsed specs and report parse failures separately"

  given
    A directory with 5 spec files where 2 fail to parse

  when analyze specs at the directory path

  then returns analysis_result
    assert analysis covers the 3 successfully parsed specs
    assert warnings list the 2 specs that failed to parse
    assert total spec count == 3


# Edge cases

behavior single-spec-analysis [edge_case]
  "Analyze a directory containing exactly one spec file"

  given
    A directory containing exactly one valid .spec file named auth.spec

  when analyze specs at the directory path

  then returns analysis_result
    assert total spec count == 1
    assert domains has exactly one domain
    assert archetype name == "generic"


behavior hierarchy-depth-calculation [edge_case]
  "Compute hierarchy depth from the longest dependency chain"

  given
    specs/a.spec depends on b
    specs/b.spec depends on c
    specs/c.spec has no dependencies
    The longest chain is a -> b -> c (depth 3)

  when analyze specs at the directory path

  then returns analysis_result
    assert hierarchy depth == 3
