spec design-export v1.0.0
title "Design Export"

description
  Writes the final design system artifacts to disk from the current
  DesignSystem state. Produces five artifacts: design-tokens.json
  (DTCG-format tokens), variables.css (CSS custom properties),
  guidelines.md (design philosophy and usage rules), components.html
  (self-contained component catalog with color swatches, type scale,
  spacing grid, and component showcase), and interface.spec (a valid
  minter spec capturing the design contract). Output defaults to
  specs/design/ and is configurable via the design.output key in
  minter.config.json. The export reads from persisted design state
  and requires a prior generate_layout call.

motivation
  The design preview is for iteration — the export is for production.
  Teams need commit-ready artifacts they can import into their build
  pipeline. DTCG-format tokens work with design tool plugins and
  JS/TS code. CSS custom properties drop into any web project.
  The component catalog (components.html) contains the design system
  documentation that was previously shown in the preview — color
  swatches, type scale visualization, spacing grid, and component
  showcase. This keeps the preview focused on the app prototype
  while still producing comprehensive design documentation as output.

nfr
  operability#deterministic-output
  reliability#no-silent-data-loss
  reliability#crash-safe-persistence


# Artifact generation — happy paths

behavior export-design-tokens-json [happy_path]
  "Write design-tokens.json in DTCG format with all color, typography, and spacing tokens"

  given
    A persisted design state with palette, type scale, and spacing

  when extract design system

  then
    assert design-tokens.json is written to the output directory
    assert file contains color tokens with primary and shade variants
    assert file contains typography tokens with font family and scale levels
    assert file contains spacing tokens with base and steps
    assert file is valid JSON


behavior export-variables-css [happy_path]
  "Write variables.css with CSS custom properties for all design tokens"

  given
    A persisted design state

  when extract design system

  then
    assert variables.css is written to the output directory
    assert file contains CSS custom property declarations
    assert file contains primary color variable
    assert file contains font family variable
    assert file contains spacing base variable


behavior export-guidelines-md [happy_path]
  "Write guidelines.md with design philosophy, usage rules, and accessibility notes"

  given
    A persisted design state with archetype "dashboard" and 3 applied decisions

  when extract design system

  then
    assert guidelines.md is written to the output directory
    assert file contains design philosophy section
    assert file contains color usage rules
    assert file contains typography hierarchy guide
    assert file contains spacing principles
    assert file contains accessibility notes with contrast ratios


behavior export-components-html [happy_path]
  "Write components.html as a self-contained design system documentation page"

  given
    A persisted design state with palette, type scale, spacing, and component styles

  when extract design system

  then
    assert components.html is written to the output directory
    assert file contains color palette swatches
    assert file contains type scale visualization
    assert file contains spacing grid
    assert file contains component showcase with button and card variants
    assert file is self-contained with inline styles


behavior export-interface-spec [happy_path]
  "Write interface.spec as a valid minter spec capturing design contracts"

  given
    A persisted design state

  when extract design system

  then
    assert interface.spec is written to the output directory
    assert file follows valid minter spec syntax
    assert file contains behaviors describing the design system contract


# Output path

behavior export-to-default-path [happy_path]
  "Write artifacts to specs/design/ when no config overrides the path"

  given
    A persisted design state
    No minter.config.json exists or it has no design.output key

  when extract design system

  then
    assert all 5 artifacts are written to specs/design/


behavior export-to-configured-path [happy_path]
  "Write artifacts to the path specified in minter.config.json design.output"

  given
    A persisted design state
    minter.config.json contains: { "design": { "output": "design-system/" } }

  when extract design system

  then
    assert all 5 artifacts are written to design-system/


behavior export-creates-output-directory [happy_path]
  "Create the output directory if it does not exist"

  given
    A persisted design state
    The output directory does not exist on disk

  when extract design system

  then
    assert the output directory is created
    assert all 5 artifacts are written to the new directory


# Artifact content quality

behavior guidelines-specific-to-archetype [happy_path]
  "Guidelines content references the detected archetype and its design rationale"

  given
    A persisted design state with archetype "dashboard"

  when extract design system

  then
    assert guidelines.md mentions the dashboard archetype
    assert guidelines.md contains rationale for layout choices


behavior guidelines-include-decisions [happy_path]
  "Guidelines document the refinement decisions that were applied"

  given
    A persisted design state with 3 applied design decisions

  when extract design system

  then
    assert guidelines.md includes a section about applied refinements
    assert the section lists the design decisions


behavior components-catalog-complete [happy_path]
  "Component catalog includes all component types with multiple states"

  given
    A persisted design state with component styles

  when extract design system

  then
    assert components.html shows button variants
    assert components.html shows input field states
    assert components.html shows card components
    assert components.html shows navigation components


# Export response

behavior export-returns-artifact-descriptions [happy_path]
  "The export response describes each artifact with file name, description, and size"

  given
    A persisted design state

  when extract design system

  then returns export_result
    assert artifacts contains 5 entries
    assert each artifact has a file name
    assert each artifact has a description
    assert each artifact has a size


behavior export-returns-design-summary [happy_path]
  "The export response includes a summary of the final design system"

  given
    A persisted design state with palette "default" and archetype "dashboard"

  when extract design system

  then returns export_result
    assert summary contains palette name
    assert summary contains archetype
    assert summary contains typography description
    assert summary contains spacing description


behavior export-returns-next-steps [happy_path]
  "The export response includes actionable next steps for using the artifacts"

  given
    A persisted design state

  when extract design system

  then returns export_result
    assert next steps is not empty
    assert next steps include guidance on importing variables.css
    assert next steps include guidance on reviewing components.html


# Error cases

behavior reject-export-without-state [error_case]
  "Return an error when no design state exists"

  given
    No design state file exists at .minter/design-state.json

  when extract design system

  then returns error
    assert error message contains "generate_layout"
    assert error message indicates no design state exists


behavior reject-corrupted-state [error_case]
  "Return an error when the design state file is corrupted"

  given
    The design state file exists but contains invalid JSON

  when extract design system

  then returns error
    assert error message contains "corrupt" or "invalid"
    assert error message suggests regenerating the design


behavior handle-write-permission-error [error_case]
  "Return an error when the output directory is not writable"

  given
    A persisted design state
    The output directory exists but is not writable

  when extract design system

  then returns error
    assert error message contains "permission"
    assert error message contains the output directory path


# Edge cases

behavior export-overwrites-existing-artifacts [edge_case]
  "Overwrite existing artifacts in the output directory without warning"

  given
    A persisted design state
    The output directory already contains a previous design-tokens.json

  when extract design system

  then
    assert design-tokens.json is overwritten with the new content
    assert all 5 artifacts reflect the current design state


behavior export-with-no-refinements [edge_case]
  "Export succeeds even when no refinements have been applied"

  given
    A persisted design state with 0 applied decisions
    The design is in its initial generated state

  when extract design system

  then returns export_result
    assert all 5 artifacts are written
    assert summary decision count == 0


behavior export-with-missing-metadata [edge_case]
  "Export succeeds when spec metadata is absent from the design state"

  given
    A persisted design state without spec metadata

  when extract design system

  then returns export_result
    assert all 5 artifacts are written
    assert guidelines.md uses generic content instead of spec-specific content


depends on design-generation >= 1.0.0
depends on config >= 1.1.0
