spec mcp-design-tools v1.0.0
title "MCP Design Tools"

description
  Three MCP tools and a guide topic that form the agent interaction
  surface for the design system generator. generate_layout analyzes
  project specs and produces an initial design with archetype
  classification, rich palette descriptions, structured layout
  configuration, and contextual refinement suggestions. refine_layout
  interprets natural language design changes across 15+ patterns —
  direct property setting, palette switching, relative color
  adjustments, typography, spacing, and border radius — returning
  before/after values, cascade descriptions, and specific guidance.
  extract_design_system exports final artifacts with descriptions
  and a design summary. The guide topic "design" teaches agents how
  to present designs to users, translate vague feedback into tool
  calls, and conduct design conversations using palette descriptions
  with product references. All tool responses include contextual
  next steps with example commands. State is persisted at the project
  root .minter/design-state.json regardless of the spec path argument.

motivation
  The quality of agent output depends entirely on what the tools give
  back. Generic responses produce generic conversations. Rich responses
  with palette descriptions, archetype reasoning, concrete refinement
  suggestions, and conversation coaching enable agents to have
  meaningful design dialogues with users. Tool descriptions must list
  available properties and palettes so agents know the vocabulary.
  Unrecognized changes must return suggestions instead of silently
  succeeding. The guide topic bridges the gap between tool mechanics
  and design communication.

nfr
  operability#deterministic-output
  operability#mcp-protocol-compliance
  operability#json-response-schema
  operability#input-schema-accuracy
  reliability#no-silent-data-loss
  reliability#crash-safe-persistence


# Tool registration

behavior register-three-design-tools [happy_path]
  "The MCP server registers generate_layout, refine_layout, and extract_design_system tools"

  given
    The MCP server has been initialized

  when tools/list

  then returns tool_list
    assert tools contains tool named "generate_layout"
    assert tools contains tool named "refine_layout"
    assert tools contains tool named "extract_design_system"
    assert each tool has a description
    assert each tool has an inputSchema


# Tool descriptions

behavior generate-layout-description-includes-response-shape [happy_path]
  "The generate_layout tool description explains what the response contains"

  given
    The MCP server has been initialized

  when tools/list

  then returns tool_list
    assert generate_layout description mentions archetype classification
    assert generate_layout description mentions layout configuration
    assert generate_layout description mentions color palette
    assert generate_layout description mentions refinement suggestions


behavior refine-layout-description-lists-patterns [happy_path]
  "The refine_layout tool description lists supported change patterns with examples"

  given
    The MCP server has been initialized

  when tools/list

  then returns tool_list
    assert refine_layout description mentions color changes
    assert refine_layout description mentions palette switching
    assert refine_layout description mentions typography changes
    assert refine_layout description mentions spacing changes
    assert refine_layout description mentions border radius changes


behavior refine-layout-change-param-lists-vocabulary [happy_path]
  "The change parameter description lists available properties and palettes"

  given
    The MCP server has been initialized

  when tools/list

  then returns tool_list
    assert refine_layout change parameter description mentions primary-color
    assert refine_layout change parameter description mentions font-family
    assert refine_layout change parameter description mentions spacing-base
    assert refine_layout change parameter description mentions available palettes


# generate_layout response

behavior generate-response-includes-analysis [happy_path]
  "The generate_layout response includes project analysis with domains and metrics"

  given
    A directory with specs across multiple subdirectories

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert response contains analysis section
    assert analysis contains project name
    assert analysis contains domains array
    assert each domain has a name and spec count
    assert analysis contains metrics with spec count and behavior count


behavior generate-response-includes-archetype-reasoning [happy_path]
  "The generate_layout response includes archetype name, confidence, and reasoning"

  given
    A directory with specs that classify as dashboard

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert response contains archetype section
    assert archetype has name
    assert archetype has confidence score
    assert archetype has reasoning string
    assert reasoning explains why the archetype was chosen


behavior generate-response-includes-rich-palette [happy_path]
  "The generate_layout response includes palette with description, semantic colors, and contrast info"

  given
    A directory with specs

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert response contains design section
    assert design contains palette
    assert palette has name
    assert palette has description
    assert palette has primary with hex value
    assert palette has semantic colors for success, warning, and error


behavior generate-response-includes-layout-config [happy_path]
  "The generate_layout response includes structured layout configuration"

  given
    A directory with specs that classify as dashboard

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert response contains design section
    assert design contains layout
    assert layout has description
    assert layout describes the archetype-specific page structure


behavior generate-response-includes-typography-and-spacing [happy_path]
  "The generate_layout response includes typography and spacing descriptions"

  given
    A directory with specs

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert design contains typography with family and description
    assert design contains spacing with base and description


behavior generate-response-includes-refinement-suggestions [happy_path]
  "The generate_layout response includes concrete refinement suggestions with example commands"

  given
    A directory with specs

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert response contains guidance section
    assert guidance contains refinement suggestions array
    assert each suggestion has a suggestion description
    assert each suggestion has a command example


behavior generate-response-includes-available-palettes [happy_path]
  "The generate_layout response lists available palettes with vibes"

  given
    A directory with specs

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert guidance contains available palettes
    assert each palette has name and vibe description
    assert available palettes includes "default"
    assert available palettes includes "slate-blue"
    assert available palettes includes "midnight"


behavior generate-response-includes-rationale [happy_path]
  "The generate_layout response includes design rationale explaining why choices were made"

  given
    A directory with specs that classify as dashboard

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert guidance contains rationale
    assert rationale is not empty
    assert rationale references the detected archetype


behavior generate-response-contextual-next-steps [happy_path]
  "The generate_layout response includes contextual next steps with conversation coaching"

  given
    A directory with specs

  when tools/call generate_layout
    path = "specs/"

  then returns tool_result
    assert response contains next steps
    assert next steps include presenting the design to the user
    assert at least one next step includes a tool reference
    assert at least one next step includes an example command


# generate_layout state persistence

behavior generate-persists-state-at-project-root [happy_path]
  "The design state is persisted at project root .minter/design-state.json regardless of spec path"

  given
    The project root is /projects/minter/
    Specs are at /projects/minter/specs/

  when tools/call generate_layout
    path = "specs/"

  then
    assert design state is written to /projects/minter/.minter/design-state.json
    assert design state is NOT written to /projects/minter/specs/.minter/design-state.json


# generate_layout errors

behavior generate-rejects-empty-directory [error_case]
  "Return an error with guidance when no spec files are found"

  given
    An empty directory with no .spec files

  when tools/call generate_layout
    path = "empty/"

  then returns tool_result
    assert isError == true
    assert error message contains "no spec" or "no .spec"
    assert error message suggests creating specs first


behavior generate-rejects-nonexistent-path [error_case]
  "Return an error when the specified path does not exist"

  given
    The path does not exist on disk

  when tools/call generate_layout
    path = "nonexistent/"

  then returns tool_result
    assert isError == true
    assert error message contains "nonexistent"


# refine_layout response — recognized changes

behavior refine-recognized-returns-before-after [happy_path]
  "A recognized change returns status applied with before and after values"

  given
    A persisted design state with primary color "#2563eb"

  when tools/call refine_layout
    change = "set primary-color to #1e40af"

  then returns tool_result
    assert status == "applied"
    assert changes contains an entry with property "primary-color"
    assert the entry has from value "#2563eb"
    assert the entry has to value "#1e40af"


behavior refine-recognized-returns-cascade [happy_path]
  "A recognized color change returns a description of cascaded updates"

  given
    A persisted design state

  when tools/call refine_layout
    change = "set primary-color to #1e40af"

  then returns tool_result
    assert changes entry includes cascaded changes description
    assert cascaded description mentions shade variants or dark mode


behavior refine-recognized-returns-snapshot [happy_path]
  "A recognized change returns a snapshot of current design values"

  given
    A persisted design state

  when tools/call refine_layout
    change = "set primary-color to #1e40af"

  then returns tool_result
    assert snapshot contains primary color
    assert snapshot contains font family
    assert snapshot contains spacing base
    assert snapshot contains decision count


behavior refine-recognized-returns-specific-guidance [happy_path]
  "A recognized change returns guidance specific to what changed"

  given
    A persisted design state

  when tools/call refine_layout
    change = "set primary-color to #1e40af"

  then returns tool_result
    assert guidance is not empty
    assert guidance references the property that changed
    assert guidance is not generic boilerplate


behavior refine-recognized-returns-contextual-next-steps [happy_path]
  "A recognized change returns next steps that suggest the logical next refinement"

  given
    A persisted design state
    The change was a color change

  when tools/call refine_layout
    change = "set primary-color to #1e40af"

  then returns tool_result
    assert next steps is not empty
    assert at least one next step suggests a different property area


# refine_layout response — unrecognized changes

behavior refine-unrecognized-returns-suggestions [error_case]
  "An unrecognized change returns status unrecognized with actionable suggestions"

  given
    A persisted design state

  when tools/call refine_layout
    change = "make it pop more"

  then returns tool_result
    assert status == "unrecognized"
    assert message describes what could not be interpreted
    assert suggestions is not empty
    assert each suggestion has a description
    assert each suggestion has a command example


behavior refine-unrecognized-lists-properties [error_case]
  "An unrecognized change returns the list of available design properties"

  given
    A persisted design state

  when tools/call refine_layout
    change = "adjust the vibe"

  then returns tool_result
    assert status == "unrecognized"
    assert available properties contains "primary-color"
    assert available properties contains "font-family"
    assert available properties contains "spacing-base"
    assert available properties contains "button-border-radius"
    assert available properties contains "container-max-width"


behavior refine-unrecognized-lists-palettes [error_case]
  "An unrecognized change returns the list of available palettes"

  given
    A persisted design state

  when tools/call refine_layout
    change = "adjust the vibe"

  then returns tool_result
    assert status == "unrecognized"
    assert available palettes contains "default"
    assert available palettes contains "slate-blue"
    assert available palettes contains "midnight"


behavior refine-unrecognized-never-changes-tokens [error_case]
  "An unrecognized change does not modify any design tokens"

  given
    A persisted design state with known token values

  when tools/call refine_layout
    change = "make the soul sing"

  then returns tool_result
    assert status == "unrecognized"
    assert the persisted design state is unchanged


# refine_layout errors

behavior refine-rejects-without-state [error_case]
  "Return an error when no design state exists"

  given
    No design state file exists

  when tools/call refine_layout
    change = "darker primary"

  then returns tool_result
    assert isError == true
    assert error message indicates generate_layout must be called first


# extract_design_system response

behavior extract-returns-artifact-descriptions [happy_path]
  "The extract response describes each artifact with file name, description, and size"

  given
    A persisted design state

  when tools/call extract_design_system

  then returns tool_result
    assert artifacts contains 5 entries
    assert each artifact has file name
    assert each artifact has description explaining its purpose
    assert each artifact has size


behavior extract-returns-design-summary [happy_path]
  "The extract response includes a summary of the final design system"

  given
    A persisted design state with palette "default" and archetype "dashboard"

  when tools/call extract_design_system

  then returns tool_result
    assert summary contains palette description
    assert summary contains archetype
    assert summary contains typography description
    assert summary contains spacing description
    assert summary contains decision count


behavior extract-returns-usage-next-steps [happy_path]
  "The extract response includes next steps for importing and using the artifacts"

  given
    A persisted design state

  when tools/call extract_design_system

  then returns tool_result
    assert next steps is not empty
    assert next steps include guidance on reviewing artifacts
    assert next steps include guidance on importing CSS variables


behavior extract-rejects-without-state [error_case]
  "Return an error when no design state exists"

  given
    No design state file exists

  when tools/call extract_design_system

  then returns tool_result
    assert isError == true
    assert error message indicates generate_layout must be called first


# Guide topic — design

behavior guide-design-topic-exists [happy_path]
  "The guide tool supports a design topic that covers the design workflow"

  given
    The MCP server has been initialized

  when tools/call guide
    topic = "design"

  then returns tool_result
    assert content covers the three-phase workflow
    assert content covers generate, refine, and extract steps


behavior guide-includes-presenting-design [happy_path]
  "The design guide includes a section on how to present design choices to users"

  given
    The MCP server has been initialized

  when tools/call guide
    topic = "design"

  then returns tool_result
    assert content includes presenting design section
    assert section coaches agents to describe the archetype first
    assert section coaches agents to explain color choices
    assert section coaches agents to ask for direction not approval


behavior guide-includes-feedback-translation [happy_path]
  "The design guide includes a table translating user feedback to tool calls"

  given
    The MCP server has been initialized

  when tools/call guide
    topic = "design"

  then returns tool_result
    assert content includes feedback translation section
    assert section maps "more professional" to a palette suggestion
    assert section maps "more breathing room" to a spacing change
    assert section maps "rounder, more modern" to a border radius change


behavior guide-includes-palette-descriptions [happy_path]
  "The design guide describes each palette with vibe and product references"

  given
    The MCP server has been initialized

  when tools/call guide
    topic = "design"

  then returns tool_result
    assert content includes palette descriptions
    assert default palette description references a known product
    assert slate-blue palette description references a known product
    assert midnight palette description references a known product


behavior guide-includes-color-psychology [happy_path]
  "The design guide includes a color psychology quick reference"

  given
    The MCP server has been initialized

  when tools/call guide
    topic = "design"

  then returns tool_result
    assert content includes color psychology section
    assert section covers blue, indigo, slate, green, and amber associations


behavior guide-includes-archetype-references [happy_path]
  "The design guide includes design system references per archetype"

  given
    The MCP server has been initialized

  when tools/call guide
    topic = "design"

  then returns tool_result
    assert content includes archetype reference section
    assert dashboard references include known dashboard products
    assert e-commerce references include known e-commerce products
    assert content references include known content products


behavior guide-includes-conversation-examples [happy_path]
  "The design guide includes example conversations showing agent behavior"

  given
    The MCP server has been initialized

  when tools/call guide
    topic = "design"

  then returns tool_result
    assert content includes conversation examples
    assert examples show how to present a first generation
    assert examples show how to handle vague feedback
    assert examples show how to handle unrecognized changes


# Edge cases

behavior generate-single-spec [edge_case]
  "generate_layout works with a directory containing exactly one spec"

  given
    A directory containing exactly one valid .spec file

  when tools/call generate_layout
    path = "single-spec/"

  then returns tool_result
    assert response contains analysis with 1 spec
    assert response contains archetype classification
    assert response contains design section


behavior refine-palette-switch-from-custom [edge_case]
  "Switching to a named palette from a custom color state replaces all colors"

  given
    A persisted design state with primary color manually set to "#ff6600"
    The palette is currently marked as "custom"

  when tools/call refine_layout
    change = "use the default palette"

  then returns tool_result
    assert status == "applied"
    assert changes palette-style to value == "default"
    assert the primary color reverts to the default palette primary


behavior refine-after-multiple-changes [edge_case]
  "Refine works correctly after multiple sequential changes"

  given
    A persisted design state
    3 previous refine_layout calls have been applied

  when tools/call refine_layout
    change = "darker primary"

  then returns tool_result
    assert status == "applied"
    assert snapshot decision count >= 4


depends on spec-analysis >= 1.0.0
depends on design-generation >= 1.0.0
depends on design-inference >= 1.0.0
depends on design-preview >= 1.0.0
depends on design-export >= 1.0.0
