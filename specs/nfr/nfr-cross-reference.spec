spec nfr-cross-reference v1.0.0
title "NFR Cross-Reference"

description
  Defines the cross-reference syntax that allows functional .spec files
  to reference constraints in .nfr files, and the validation rules
  minter enforces on those references. Covers spec-level and behavior-level
  nfr sections, anchor resolution, override validation, and structural rules.

motivation
  NFR references connect functional specs to non-functional requirements.
  Without validated cross-references, NFR constraints could be referenced
  incorrectly — pointing at missing files, nonexistent constraints, or
  using invalid overrides. This spec ensures all references are mechanically
  verified.

# Spec-level nfr section parsing

behavior parse-spec-level-whole-file-ref [happy_path]
  "Parse a spec-level nfr section with a whole-file category reference"

  given
    A .spec file with:
    nfr
      security

  when parse

  then
    assert nfr reference count == 1
    assert nfr reference 1 category == "security"
    assert nfr reference 1 type == "whole_file"


behavior parse-spec-level-anchor-ref [happy_path]
  "Parse a spec-level nfr section with an anchor reference"

  given
    A .spec file with:
    nfr
      reliability#completeness

  when parse

  then
    assert nfr reference count == 1
    assert nfr reference 1 category == "reliability"
    assert nfr reference 1 anchor == "completeness"


behavior parse-spec-level-mixed-refs [happy_path]
  "Parse a spec-level nfr section with both whole-file and anchor references"

  given
    A .spec file with:
    nfr
      security
      performance
      reliability#completeness

  when parse

  then
    assert nfr reference count == 3
    assert nfr reference 1 type == "whole_file"
    assert nfr reference 2 type == "whole_file"
    assert nfr reference 3 type == "anchor"


behavior parse-spec-level-nfr-optional [edge_case]
  "A spec with no nfr section is valid"

  given
    A .spec file with a valid header and behaviors but no nfr section

  when parse

  then
    assert nfr references count == 0

  then emits process_exit
    assert code == 0


behavior parse-spec-level-nfr-position [happy_path]
  "The nfr section appears after motivation and before the first behavior"

  given
    A .spec file with nfr section placed between motivation and the
    first behavior declaration

  when parse

  then
    assert nfr section is_present
    assert nfr section position is after motivation
    assert nfr section position is before first behavior


behavior reject-spec-level-override [error_case]
  "Reject threshold overrides in the spec-level nfr section"

  given
    A .spec file with:
    nfr
      performance#api-response-time < 500ms

  when parse

  then emits stderr
    assert output contains "override"
    assert output contains "spec-level"

  then emits process_exit
    assert code == 1


# Behavior-level nfr section parsing

behavior parse-behavior-level-anchor-ref [happy_path]
  "Parse a behavior-level nfr section with an anchor reference"

  given
    A behavior block with:
    nfr
      performance#data-freshness

  when parse

  then
    assert behavior nfr reference count == 1
    assert behavior nfr reference 1 category == "performance"
    assert behavior nfr reference 1 anchor == "data-freshness"


behavior parse-behavior-level-override [happy_path]
  "Parse a behavior-level nfr reference with a threshold override"

  given
    A behavior block with:
    nfr
      performance#api-response-time < 500ms

  when parse

  then
    assert behavior nfr reference 1 category == "performance"
    assert behavior nfr reference 1 anchor == "api-response-time"
    assert behavior nfr reference 1 override operator == "<"
    assert behavior nfr reference 1 override value == "500ms"


behavior parse-behavior-level-multiple-refs [happy_path]
  "Parse a behavior-level nfr section with multiple references"

  given
    A behavior block with:
    nfr
      performance#api-response-time < 500ms
      reliability#completeness >= 100%

  when parse

  then
    assert behavior nfr reference count == 2
    assert behavior nfr reference 1 anchor == "api-response-time"
    assert behavior nfr reference 2 anchor == "completeness"


behavior parse-behavior-level-nfr-position [happy_path]
  "The behavior-level nfr section appears after description and before given"

  given
    A behavior block with nfr section placed between the quoted
    description and the given section

  when parse

  then
    assert behavior nfr section is_present
    assert behavior nfr section position is after description
    assert behavior nfr section position is before given


behavior parse-behavior-level-nfr-optional [edge_case]
  "A behavior with no nfr section is valid"

  given
    A behavior block with given, when, and then but no nfr section

  when parse

  then
    assert behavior nfr references count == 0


# Cross-validation rule 1: Category exists

behavior resolve-category-to-nfr-file [happy_path]
  "Resolve a referenced category to a .nfr file declaring that category"

  given
    A .spec file with nfr section referencing performance
    A .nfr file in the same directory tree declaring: nfr performance v1.0.0

  when minter validate specs/

  then emits process_exit
    assert code == 0


behavior reject-missing-nfr-category-file [error_case]
  "Reject when a referenced NFR category has no corresponding .nfr file"

  given
    A .spec file with nfr section referencing performance
    No .nfr file in the directory tree declares category performance

  when minter validate specs/

  then emits stderr
    assert output contains "performance"
    assert output contains "not found" or "missing"

  then emits process_exit
    assert code == 1


# Cross-validation rule 2: Anchor exists

behavior resolve-anchor-to-constraint [happy_path]
  "Resolve a #constraint-name anchor to a constraint in the .nfr file"

  given
    A .spec file referencing performance#api-response-time
    The performance .nfr file contains constraint api-response-time [metric]

  when minter validate specs/

  then emits process_exit
    assert code == 0


behavior reject-missing-anchor [error_case]
  "Reject when a #constraint-name does not match any constraint in the .nfr file"

  given
    A .spec file referencing performance#nonexistent
    The performance .nfr file has no constraint named nonexistent

  when minter validate specs/

  then emits stderr
    assert output contains "nonexistent"
    assert output contains "performance"

  then emits process_exit
    assert code == 1


# Cross-validation rule 3: Containment

behavior containment-satisfied [happy_path]
  "Pass when behavior-level categories are declared in spec-level nfr"

  given
    A .spec file with:
    nfr
      performance
    And a behavior with:
    nfr
      performance#api-response-time

  when minter validate specs/

  then emits process_exit
    assert code == 0


behavior reject-containment-violation [error_case]
  "Reject when a behavior references a category not in spec-level nfr"

  given
    A .spec file with:
    nfr
      performance
    And a behavior with:
    nfr
      reliability#completeness

  when minter validate specs/

  then emits stderr
    assert output contains "reliability"
    assert output contains "containment" or "spec-level"

  then emits process_exit
    assert code == 1


# Cross-validation rule 4: Overridable check

behavior override-allowed-on-overridable-yes [happy_path]
  "Allow override on a constraint marked overridable yes"

  given
    A .spec behavior referencing performance#api-response-time < 500ms
    The constraint api-response-time has overridable yes

  when minter validate specs/

  then emits process_exit
    assert code == 0


behavior reject-override-on-overridable-no [error_case]
  "Reject override on a constraint marked overridable no"

  given
    A .spec behavior referencing security#tenant-isolation < 500ms
    The constraint tenant-isolation has overridable no

  when minter validate specs/

  then emits stderr
    assert output contains "tenant-isolation"
    assert output contains "overridable" or "not overridable"

  then emits process_exit
    assert code == 1


# Cross-validation rule 5: Metric only

behavior override-allowed-on-metric [happy_path]
  "Allow override on a metric constraint"

  given
    A .spec behavior referencing performance#api-response-time < 500ms
    The constraint api-response-time is type [metric] with overridable yes

  when minter validate specs/

  then emits process_exit
    assert code == 0


behavior reject-override-on-rule [error_case]
  "Reject override on a rule constraint"

  given
    A .spec behavior referencing performance#no-n-plus-one < 5
    The constraint no-n-plus-one is type [rule]

  when minter validate specs/

  then emits stderr
    assert output contains "no-n-plus-one"
    assert output contains "rule" or "metric"

  then emits process_exit
    assert code == 1


# Cross-validation rule 6: Same operator

behavior override-same-operator [happy_path]
  "Allow override when the operator matches the original threshold"

  given
    A .spec behavior referencing performance#api-response-time < 500ms
    The constraint api-response-time has threshold < 1s

  when minter validate specs/

  then emits process_exit
    assert code == 0


behavior reject-override-mismatched-operator [error_case]
  "Reject override when the operator does not match the original threshold"

  given
    A .spec behavior referencing performance#api-response-time > 500ms
    The constraint api-response-time has threshold < 1s

  when minter validate specs/

  then emits stderr
    assert output contains "operator"
    assert output contains "api-response-time"

  then emits process_exit
    assert code == 1


# Cross-validation rule 7: Stricter value

behavior override-stricter-value [happy_path]
  "Allow override when the value is stricter than the default"

  given
    A .spec behavior referencing performance#api-response-time < 500ms
    The constraint api-response-time has threshold < 1s
    500ms is stricter (smaller) than 1s for the < operator

  when minter validate specs/

  then emits process_exit
    assert code == 0


behavior reject-override-relaxed-value [error_case]
  "Reject override when the value is more relaxed than the default"

  given
    A .spec behavior referencing performance#api-response-time < 2s
    The constraint api-response-time has threshold < 1s
    2s is more relaxed (larger) than 1s for the < operator

  when minter validate specs/

  then emits stderr
    assert output contains "stricter" or "relaxed"
    assert output contains "api-response-time"

  then emits process_exit
    assert code == 1


# Cross-validation rule 8: No spec-level overrides (covered above)
# See: reject-spec-level-override

# Cross-validation rule 9: No behavior-level whole-file references

behavior reject-behavior-level-whole-file-ref [error_case]
  "Reject whole-file references in a behavior-level nfr section"

  given
    A behavior block with:
    nfr
      performance

  when parse

  then emits stderr
    assert output contains "whole-file"
    assert output contains "behavior"

  then emits process_exit
    assert code == 1


behavior accept-behavior-level-anchor-only [happy_path]
  "Accept behavior-level nfr with only anchor references"

  given
    A behavior block with:
    nfr
      performance#api-response-time

  when parse

  then
    assert behavior nfr reference 1 type == "anchor"


depends on spec-grammar >= 1.1.0
depends on nfr-grammar >= 1.0.0
