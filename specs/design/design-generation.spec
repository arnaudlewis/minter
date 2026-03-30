spec design-generation v1.0.0
title "Design Generation"

description
  Takes an analysis result from spec-analysis and taste defaults from
  design-taste, and produces a complete DesignSystem data structure. The
  DesignSystem includes a resolved color palette with shades and semantic
  colors, a type scale, a spacing grid, component styles, and a
  structured layout configuration. Each archetype produces a different
  layout — dashboard gets a sidebar and metric cards, e-commerce gets a
  product grid, content gets a reading-width container, form-heavy gets
  a wizard with progress indicator. The layout configuration contains
  structured data (sidebar position, header title, content sections with
  kinds and widths), not just section name strings. Spec metadata from
  the analysis is carried through to populate the preview with real
  content — nav items from spec names, metrics from real counts.

motivation
  The generation step bridges analysis (what the project IS) and
  presentation (what the design LOOKS LIKE). Without structured layout
  configuration, the frontend has to guess how to arrange components.
  Without spec metadata, the preview shows placeholder text instead of
  real project data. The DesignSystem is a pure data structure — it
  never writes files or renders HTML. That separation keeps generation
  testable and lets preview and export consume the same tokens.

nfr
  operability#deterministic-output
  reliability#no-silent-data-loss


# Layout generation per archetype

behavior generate-dashboard-sidebar-and-header [happy_path]
  "Dashboard archetype produces a left sidebar with nav items and a header with project name"

  given
    The analysis result has archetype "dashboard"
    Spec metadata includes project name "Minter" and spec names

  when generate design system from the analysis result

  then returns design_system
    assert layout has a sidebar
    assert sidebar position == "left"
    assert sidebar nav items are derived from spec names
    assert layout has a header
    assert header title matches the project name


behavior generate-dashboard-content-sections [happy_path]
  "Dashboard archetype includes metric cards and data table in the content area"

  given
    The analysis result has archetype "dashboard"

  when generate design system from the analysis result

  then returns design_system
    assert layout content sections include "metric-cards"
    assert layout content sections include "data-table"


behavior generate-ecommerce-layout [happy_path]
  "E-commerce archetype produces a top navigation, hero section, and product grid"

  given
    The analysis result has archetype "e-commerce"

  when generate design system from the analysis result

  then returns design_system
    assert layout has a header with navigation
    assert layout does not have a sidebar
    assert layout content sections include "product-grid"


behavior generate-content-layout [happy_path]
  "Content archetype produces a reading-width container with article typography"

  given
    The analysis result has archetype "content"

  when generate design system from the analysis result

  then returns design_system
    assert layout does not have a sidebar
    assert layout has a header
    assert layout content sections include "article-body"
    assert container max width is present


behavior generate-form-heavy-layout [happy_path]
  "Form-heavy archetype produces a wizard with progress indicator and form sections"

  given
    The analysis result has archetype "form-heavy"

  when generate design system from the analysis result

  then returns design_system
    assert layout does not have a sidebar
    assert layout has a header
    assert layout content sections include "wizard-form"
    assert layout content sections include "progress-indicator"


behavior generate-generic-layout [happy_path]
  "Generic archetype infers layout from spec structure rather than using a fixed template"

  given
    The analysis result has archetype "generic"
    The project has 10 specs across 3 domains

  when generate design system from the analysis result

  then returns design_system
    assert layout is present
    assert layout content sections is not empty
    assert layout content sections are derived from spec structure


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
  "Different archetypes may suggest different default palettes"

  given
    The analysis result has archetype "dashboard"

  when generate design system from the analysis result

  then returns design_system
    assert palette is resolved from the taste layer
    assert palette description is present


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
  "The design system carries spec metadata for preview content population"

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


behavior sidebar-nav-from-spec-names [happy_path]
  "Sidebar navigation items are derived from spec names in the metadata"

  given
    The analysis result has archetype "dashboard"
    Spec metadata includes spec names: validate-command, watch-command,
    format-command, graph-command, coverage-command

  when generate design system from the analysis result

  then returns design_system
    assert sidebar nav items is not empty
    assert sidebar nav items count > 0


# Content sections

behavior content-sections-have-structure [happy_path]
  "Each content section has a kind, title, and width"

  given
    The analysis result has archetype "dashboard"

  when generate design system from the analysis result

  then returns design_system
    assert each content section has a kind
    assert each content section has a title
    assert each content section has a width


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
    assert layout is present
    assert palette is present
    assert spec metadata is absent or empty
    assert sidebar nav items use fallback labels


# Edge cases

behavior empty-domains-layout [edge_case]
  "Generate layout with default content sections when domains list is empty"

  given
    The analysis result has archetype "generic"
    The domains list is empty

  when generate design system from the analysis result

  then returns design_system
    assert layout content sections is not empty
    assert layout content sections contain at least one default section


behavior single-domain-layout [edge_case]
  "A project with a single domain still produces a valid layout"

  given
    The analysis result has archetype "generic"
    One domain with 3 specs

  when generate design system from the analysis result

  then returns design_system
    assert layout is present
    assert layout content sections is not empty


depends on spec-analysis >= 1.0.0
depends on design-taste >= 1.0.0
