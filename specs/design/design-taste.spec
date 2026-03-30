spec design-taste v1.0.0
title "Design Taste Layer"

description
  Curated design defaults that encode opinionated aesthetic choices.
  Provides three named palettes (default, slate-blue, midnight), each
  with a human-readable description, vibe keywords, and reference
  products that agents can relay to users. Enforces WCAG AA contrast
  compliance on all color pairings. Defines a type scale using a
  configurable ratio, a spacing grid based on a base unit, and animation
  defaults for transitions. The taste layer is the single source of
  aesthetic truth — generation and inference both read from it.

motivation
  Design tokens without opinion produce generic output. The taste layer
  injects the aesthetic judgment that makes generated designs feel
  intentional rather than mechanical. Palette descriptions with vibes
  and reference products (think GitHub, think Linear, think Figma)
  give agents the vocabulary to explain design choices to users in
  relatable terms. WCAG enforcement prevents the system from producing
  inaccessible color combinations regardless of user refinements.

nfr
  operability#deterministic-output
  reliability#no-silent-data-loss


# Palette definitions

behavior default-palette-properties [happy_path]
  "The default palette provides professional blue tones with description and reference"

  given
    No palette preference has been specified

  when the taste layer resolves the default palette

  then returns palette
    assert name == "default"
    assert primary hex is a valid hex color
    assert description is not empty
    assert description references trust or professional or precision
    assert palette includes 10 shade variants of the primary


behavior slate-blue-palette-properties [happy_path]
  "The slate-blue palette provides a dark, data-dense aesthetic"

  given
    The palette preference is "slate-blue"

  when the taste layer resolves the palette

  then returns palette
    assert name == "slate-blue"
    assert primary hex is a valid hex color
    assert description is not empty
    assert description references data or technical or developer
    assert palette includes 10 shade variants of the primary


behavior midnight-palette-properties [happy_path]
  "The midnight palette provides a contemporary indigo aesthetic with pink accent"

  given
    The palette preference is "midnight"

  when the taste layer resolves the palette

  then returns palette
    assert name == "midnight"
    assert primary hex is a valid hex color
    assert accent hex is a valid hex color
    assert description is not empty
    assert description references modern or creative or contemporary
    assert palette includes 10 shade variants of the primary


behavior palette-includes-semantic-colors [happy_path]
  "Every palette includes semantic colors for success, warning, error, and info"

  given
    Any palette is resolved

  when the taste layer resolves the palette

  then returns palette
    assert semantic colors contains success
    assert semantic colors contains warning
    assert semantic colors contains error
    assert semantic colors contains info
    assert each semantic color is a valid hex value


behavior palette-includes-dark-mode [happy_path]
  "Every palette includes a dark mode variant with inverted backgrounds and adjusted foregrounds"

  given
    Any palette is resolved

  when the taste layer resolves the palette

  then returns palette
    assert dark mode variant is present
    assert dark mode background is darker than light mode background
    assert dark mode foreground is lighter than light mode foreground


# WCAG AA compliance

behavior wcag-aa-primary-on-white [happy_path]
  "Primary color on white background meets WCAG AA contrast ratio"

  given
    Any palette is resolved

  when the taste layer checks contrast compliance

  then
    assert primary color on white has contrast ratio >= 4.5


behavior wcag-aa-semantic-colors [happy_path]
  "All semantic colors on their respective backgrounds meet WCAG AA"

  given
    Any palette is resolved with semantic colors

  when the taste layer checks contrast compliance

  then
    assert each semantic color on white has contrast ratio >= 4.5


behavior wcag-aa-dark-mode [happy_path]
  "Dark mode foreground on dark mode background meets WCAG AA"

  given
    Any palette with dark mode variant is resolved

  when the taste layer checks contrast compliance

  then
    assert dark mode foreground on background has contrast ratio >= 4.5


# Type scale

behavior type-scale-defaults [happy_path]
  "The default type scale uses a Major Third ratio with 7 levels"

  given
    No type scale overrides are specified

  when the taste layer resolves the type scale

  then returns type_scale
    assert ratio == 1.25
    assert level count == 7
    assert body size == 16
    assert smallest level is smaller than body size
    assert largest level is larger than body size


behavior type-scale-levels-progressive [happy_path]
  "Each type scale level is progressively larger than the previous"

  given
    The type scale is resolved with any ratio

  when the taste layer resolves the type scale

  then returns type_scale
    assert each level size is strictly greater than the previous level


# Spacing grid

behavior spacing-grid-defaults [happy_path]
  "The default spacing grid uses a 4px base with 8 steps"

  given
    No spacing overrides are specified

  when the taste layer resolves the spacing grid

  then returns spacing_scale
    assert base == 4
    assert step count == 8
    assert first step equals base value
    assert each step is larger than the previous


behavior spacing-grid-ratio [happy_path]
  "Spacing steps grow by a consistent ratio"

  given
    The spacing grid is resolved with base 4 and ratio 1.5

  when the taste layer resolves the spacing grid

  then returns spacing_scale
    assert ratio == 1.5
    assert step count == 8


# Animation defaults

behavior animation-transition-defaults [happy_path]
  "Default transition duration and easing for interactive elements"

  given
    No animation overrides are specified

  when the taste layer resolves animation defaults

  then returns animation_defaults
    assert transition duration is a positive number in milliseconds
    assert easing function is not empty


# Component style defaults

behavior component-border-radius-default [happy_path]
  "Default border radius for buttons and cards"

  given
    No component style overrides are specified

  when the taste layer resolves component style defaults

  then returns component_styles
    assert button border radius is a non-negative number
    assert card border radius is a non-negative number


# Error cases

behavior reject-unknown-palette-name [error_case]
  "Return an error when the requested palette name is not recognized"

  given
    The palette preference is "ocean-breeze" which does not exist

  when the taste layer resolves the palette

  then returns error
    assert error message contains "ocean-breeze"
    assert error message contains available palette names


behavior reject-invalid-base-size [error_case]
  "Return an error when a spacing base of zero or negative is requested"

  given
    A spacing base override of 0

  when the taste layer resolves the spacing grid

  then returns error
    assert error message contains "base" or "spacing"
    assert error message indicates the value must be positive


# Edge cases

behavior palette-shade-ordering [edge_case]
  "Palette shades are ordered from lightest (50) to darkest (900)"

  given
    Any palette is resolved

  when the taste layer resolves shade variants

  then returns palette
    assert shade 50 is lighter than shade 100
    assert shade 100 is lighter than shade 200
    assert shade 800 is lighter than shade 900


behavior complementary-color-computed [edge_case]
  "Each palette computes a complementary color from the primary"

  given
    Any palette is resolved

  when the taste layer resolves the palette

  then returns palette
    assert complementary color is present
    assert complementary color differs from primary
