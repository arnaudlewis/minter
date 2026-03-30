spec design-generation v2.0.0
title "Design Generation"

description
  Takes an analysis result from spec-analysis and taste defaults from
  design-taste, and produces a complete DesignSystem data structure. The
  DesignSystem includes a resolved color palette with shades and semantic
  colors, a type scale, a spacing grid, and component styles. The
  archetype influences default token choices — dashboard projects
  default to a data-dense palette, e-commerce to warmer tones — but
  does not determine page layout. The preview always renders a
  component showcase page. Spec metadata from the analysis is carried
  through so the showcase can display real content: spec names in
  navigation, behavior counts in metric cards, spec entries in data
  tables.

motivation
  The generation step bridges analysis (what the project IS) and
  presentation (what the design LOOKS LIKE). The DesignSystem is a
  pure data structure — it never writes files or renders HTML. That
  separation keeps generation testable and lets preview and export
  consume the same tokens. The archetype drives token defaults rather
  than layout structure, which simplifies generation and eliminates
  the maintenance burden of archetype-specific templates.

nfr
  operability#deterministic-output
  reliability#no-silent-data-loss


# Palette application

behavior apply-default-palette [happy_path]
  "Apply the default palette from the taste layer when no preference is specified"

  given
    The analysis result has no palette preference
    The taste layer provides the default palette

  when generate design system from the analysis result

  then returns design_system
    assert palette name == "default"
    assert palette primary hex is a valid hex color
    assert palette has 10 shade variants
    assert palette has semantic colors
    assert palette has dark mode variant


behavior apply-archetype-palette-hint [happy_path]
  "Different archetypes suggest different default palettes"

  given
    The analysis result has archetype "dashboard"

  when generate design system from the analysis result

  then returns design_system
    assert palette is resolved from the taste layer
    assert palette description is present


behavior apply-ecommerce-palette-hint [happy_path]
  "E-commerce archetype suggests a warmer default palette"

  given
    The analysis result has archetype "e-commerce"

  when generate design system from the analysis result

  then returns design_system
    assert palette is resolved from the taste layer
    assert palette is not the same as the dashboard default


# Type scale and spacing

behavior generate-type-scale [happy_path]
  "The type scale is resolved from taste defaults and included in the design system"

  given
    The taste layer provides a type scale with ratio 1.25 and 7 levels

  when generate design system from the analysis result

  then returns design_system
    assert type scale ratio == 1.25
    assert type scale level count == 7
    assert type scale body size is present


behavior generate-spacing-scale [happy_path]
  "The spacing grid is resolved from taste defaults and included in the design system"

  given
    The taste layer provides a spacing grid with base 4 and 8 steps

  when generate design system from the analysis result

  then returns design_system
    assert spacing base == 4
    assert spacing step count == 8


# Component styles

behavior generate-component-styles [happy_path]
  "Component styles include border radius, shadow, and padding defaults"

  given
    The taste layer provides component style defaults

  when generate design system from the analysis result

  then returns design_system
    assert component styles has button border radius
    assert component styles has card border radius


# Spec metadata pass-through

behavior include-spec-metadata [happy_path]
  "The design system carries spec metadata for showcase content population"

  given
    The analysis result includes spec metadata with:
    project name "Minter", 29 specs, 653 behaviors, 14 entities
    Domains: Core Commands (11 specs), Grammar (2 specs), MCP (3 specs)

  when generate design system from the analysis result

  then returns design_system
    assert spec metadata is present
    assert spec metadata project name == "Minter"
    assert spec metadata total spec count == 29
    assert spec metadata total behavior count == 653
    assert spec metadata domains is not empty


behavior nav-items-from-spec-names [happy_path]
  "Navigation items for the showcase are derived from spec names in the metadata"

  given
    Spec metadata includes spec names: validate-command, watch-command,
    format-command, graph-command, coverage-command

  when generate design system from the analysis result

  then returns design_system
    assert spec metadata spec names is not empty
    assert spec metadata spec names count > 0


# Archetype for guidance

behavior include-archetype-for-guidance [happy_path]
  "The design system includes the detected archetype for agent guidance"

  given
    The analysis result has archetype "dashboard" with confidence 0.82

  when generate design system from the analysis result

  then returns design_system
    assert archetype is present
    assert archetype == "dashboard"


# Component showcase list

behavior include-showcase-components [happy_path]
  "The design system includes the list of components for the showcase"

  given
    The analysis result has an archetype

  when generate design system from the analysis result

  then returns design_system
    assert component list includes "navigation"
    assert component list includes "metric-cards"
    assert component list includes "data-table"
    assert component list includes "buttons"
    assert component list includes "form-elements"
    assert component list includes "alerts-badges"
    assert component list includes "typography"
    assert component list includes "color-palette"
    assert component list includes "spacing"


# Error cases

behavior reject-missing-analysis [error_case]
  "Return an error when no analysis result is provided"

  given
    No analysis result is available

  when generate design system

  then returns error
    assert error message contains "analysis"


behavior handle-missing-spec-metadata [edge_case]
  "Generate a valid design system even when spec metadata is absent"

  given
    The analysis result has an archetype but no spec metadata

  when generate design system from the analysis result

  then returns design_system
    assert palette is present
    assert component list is present
    assert spec metadata is absent or empty


# Edge cases

behavior empty-domains-fallback [edge_case]
  "Generate valid tokens when domains list is empty"

  given
    The analysis result has archetype "generic"
    The domains list is empty

  when generate design system from the analysis result

  then returns design_system
    assert palette is present
    assert component list is not empty


behavior single-domain-project [edge_case]
  "A project with a single domain still produces a valid design system"

  given
    The analysis result has archetype "generic"
    One domain with 3 specs

  when generate design system from the analysis result

  then returns design_system
    assert palette is present
    assert spec metadata is present


depends on spec-analysis >= 1.0.0
depends on design-taste >= 1.0.0
