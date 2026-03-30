spec design-inference v1.0.0
title "Design Inference"

description
  Interprets natural language design change descriptions and applies
  cascading micro-decisions to the current DesignSystem state. Supports
  direct property setting (set primary-color to #1e40af), palette
  switching (use the midnight palette), relative color adjustments
  (darker, lighter, warmer, cooler, more muted, more vibrant), typography
  changes (font-family, type scale ratio), spacing changes (spacing-base,
  more/less breathing room), border radius changes (rounder, sharper),
  and container width changes. Each change cascades automatically —
  changing primary-color regenerates all shades, dark mode, and semantic
  colors. When a change description cannot be interpreted, the system
  returns an unrecognized result with actionable suggestions instead
  of silently succeeding with no effect.

motivation
  Users describe design preferences in natural language, not token
  values. The inference engine translates phrases like "more
  professional", "too corporate", or "rounder corners" into concrete
  token changes. Relative adjustments (darker/lighter) must operate
  on the CURRENT state — hardcoding a fixed hex for "darker" regardless
  of the current primary defeats the purpose of iterative refinement.
  Returning clear unrecognized results prevents the agent from thinking
  a change was applied when nothing happened.

nfr
  operability#deterministic-output
  reliability#no-silent-data-loss


# Direct property setting

behavior set-primary-color [happy_path]
  "Interpret 'set primary-color to #1e40af' as a direct color change"

  given
    A current DesignSystem with primary color "#2563eb"

  when interpret change "set primary-color to #1e40af"

  then returns change_result
    assert status == "applied"
    assert changes contains property "primary-color"
    assert changes primary-color from value == "#2563eb"
    assert changes primary-color to value == "#1e40af"


behavior set-font-family [happy_path]
  "Interpret 'set font-family to Inter' as a typography change"

  given
    A current DesignSystem with font family "system-ui"

  when interpret change "set font-family to Inter"

  then returns change_result
    assert status == "applied"
    assert changes contains property "font-family"
    assert changes font-family to value == "Inter"


behavior set-spacing-base [happy_path]
  "Interpret 'set spacing-base to 8px' as a spacing change"

  given
    A current DesignSystem with spacing base 4

  when interpret change "set spacing-base to 8px"

  then returns change_result
    assert status == "applied"
    assert changes contains property "spacing-base"
    assert changes spacing-base to value == "8px"


behavior set-button-border-radius [happy_path]
  "Interpret 'set button-border-radius to 8px' as a component style change"

  given
    A current DesignSystem with button border radius 4

  when interpret change "set button-border-radius to 8px"

  then returns change_result
    assert status == "applied"
    assert changes contains property "button-border-radius"


behavior set-container-max-width [happy_path]
  "Interpret 'set container-max-width to 1200px' as a layout change"

  given
    A current DesignSystem with container max width "1024px"

  when interpret change "set container-max-width to 1200px"

  then returns change_result
    assert status == "applied"
    assert changes contains property "container-max-width"


# Palette switching

behavior switch-palette [happy_path]
  "Interpret 'use the midnight palette' as a full palette regeneration"

  given
    A current DesignSystem with palette "default"

  when interpret change "use the midnight palette"

  then returns change_result
    assert status == "applied"
    assert changes contains property "palette-style"
    assert changes palette-style to value == "midnight"
    assert cascaded changes include shade regeneration
    assert cascaded changes include dark mode update


behavior switch-palette-alternate-phrasing [happy_path]
  "Interpret 'switch to slate-blue' as a palette switch"

  given
    A current DesignSystem with palette "default"

  when interpret change "switch to slate-blue"

  then returns change_result
    assert status == "applied"
    assert changes contains property "palette-style"
    assert changes palette-style to value == "slate-blue"


# Relative color adjustments

behavior darker-primary [happy_path]
  "Interpret 'darker primary' as a relative lightness decrease on the current primary"

  given
    A current DesignSystem with primary color "#2563eb"

  when interpret change "darker primary"

  then returns change_result
    assert status == "applied"
    assert changes contains property "primary-color"
    assert changes primary-color to value differs from "#2563eb"
    assert new primary is darker than the original


behavior lighter-primary [happy_path]
  "Interpret 'lighter primary' as a relative lightness increase on the current primary"

  given
    A current DesignSystem with primary color "#1e40af"

  when interpret change "lighter primary"

  then returns change_result
    assert status == "applied"
    assert changes primary-color to value is lighter than "#1e40af"


behavior warmer-tones [happy_path]
  "Interpret 'warmer tones' as a hue shift toward red/orange"

  given
    A current DesignSystem with a blue-based primary color

  when interpret change "warmer tones"

  then returns change_result
    assert status == "applied"
    assert changes contains property "primary-color"


behavior cooler-tones [happy_path]
  "Interpret 'cooler tones' as a hue shift toward blue"

  given
    A current DesignSystem with a warm-toned primary color

  when interpret change "cooler tones"

  then returns change_result
    assert status == "applied"
    assert changes contains property "primary-color"


behavior more-muted [happy_path]
  "Interpret 'more muted colors' as a saturation decrease"

  given
    A current DesignSystem with a saturated primary color

  when interpret change "more muted colors"

  then returns change_result
    assert status == "applied"
    assert changes contains property "primary-color"


behavior more-vibrant [happy_path]
  "Interpret 'more vibrant colors' as a saturation increase"

  given
    A current DesignSystem with a desaturated primary color

  when interpret change "more vibrant colors"

  then returns change_result
    assert status == "applied"
    assert changes contains property "primary-color"


# Typography adjustments

behavior larger-headings [happy_path]
  "Interpret 'larger headings' as a type scale ratio increase"

  given
    A current DesignSystem with type scale ratio 1.25

  when interpret change "larger headings"

  then returns change_result
    assert status == "applied"
    assert changes contains property related to type scale


behavior font-shorthand [happy_path]
  "Interpret 'use Inter' without the full 'set font-family to' prefix"

  given
    A current DesignSystem with font family "system-ui"

  when interpret change "use Inter"

  then returns change_result
    assert status == "applied"
    assert changes contains property "font-family"
    assert changes font-family to value == "Inter"


# Spacing adjustments

behavior more-breathing-room [happy_path]
  "Interpret 'more breathing room' as a spacing base increase"

  given
    A current DesignSystem with spacing base 4

  when interpret change "more breathing room"

  then returns change_result
    assert status == "applied"
    assert changes contains property "spacing-base"
    assert new spacing base is larger than 4


behavior tighter-layout [happy_path]
  "Interpret 'tighter layout' as a spacing base decrease"

  given
    A current DesignSystem with spacing base 8

  when interpret change "tighter layout"

  then returns change_result
    assert status == "applied"
    assert changes contains property "spacing-base"
    assert new spacing base is smaller than 8


# Border radius adjustments

behavior rounder-corners [happy_path]
  "Interpret 'rounder corners' as a border radius increase"

  given
    A current DesignSystem with button border radius 4

  when interpret change "rounder corners"

  then returns change_result
    assert status == "applied"
    assert changes contains property "button-border-radius"
    assert new border radius is larger than 4


behavior sharper-corners [happy_path]
  "Interpret 'sharper corners' as a border radius decrease"

  given
    A current DesignSystem with button border radius 8

  when interpret change "sharper corners"

  then returns change_result
    assert status == "applied"
    assert changes contains property "button-border-radius"
    assert new border radius is smaller than 8


# Cascade behavior

behavior primary-color-cascades [happy_path]
  "Changing primary color regenerates all shade variants, dark mode, and semantic colors"

  given
    A current DesignSystem with primary color "#2563eb"

  when interpret change "set primary-color to #1e40af"

  then returns change_result
    assert cascaded changes is not empty
    assert cascaded changes include shade variant regeneration
    assert cascaded changes include dark mode palette update


behavior palette-switch-cascades [happy_path]
  "Switching palette replaces the entire color system including shades and semantic colors"

  given
    A current DesignSystem with palette "default"

  when interpret change "use the midnight palette"

  then returns change_result
    assert cascaded changes include shade regeneration
    assert cascaded changes include semantic color update
    assert cascaded changes include dark mode update


# Snapshot after change

behavior snapshot-after-change [happy_path]
  "The result includes a snapshot of key design values after the change"

  given
    A current DesignSystem

  when any recognized change is applied

  then returns change_result
    assert snapshot contains primary color
    assert snapshot contains font family
    assert snapshot contains spacing base
    assert snapshot contains decision count


# Unrecognized changes

behavior unrecognized-returns-suggestions [error_case]
  "Return suggestions when the change description cannot be interpreted"

  given
    A current DesignSystem

  when interpret change "make it pop more"

  then returns change_result
    assert status == "unrecognized"
    assert input == "make it pop more"
    assert message describes what could not be interpreted
    assert suggestions is not empty
    assert each suggestion has a description
    assert each suggestion has a command example


behavior unrecognized-lists-available-properties [error_case]
  "The unrecognized result includes available properties and palettes"

  given
    A current DesignSystem

  when interpret change "adjust the vibe"

  then returns change_result
    assert status == "unrecognized"
    assert available properties contains "primary-color"
    assert available properties contains "font-family"
    assert available properties contains "spacing-base"
    assert available palettes contains "default"
    assert available palettes contains "slate-blue"
    assert available palettes contains "midnight"


behavior unrecognized-never-silently-succeeds [error_case]
  "An unrecognized change never returns status applied with no actual token change"

  given
    A current DesignSystem with known token values

  when interpret change "make the soul sing"

  then returns change_result
    assert status == "unrecognized"
    assert design system tokens are unchanged


# Edge cases

behavior hex-in-string-interpreted-as-primary [edge_case]
  "A bare hex value in the change string is interpreted as a primary color change"

  given
    A current DesignSystem

  when interpret change "#1e40af"

  then returns change_result
    assert status == "applied"
    assert changes contains property "primary-color"
    assert changes primary-color to value == "#1e40af"


behavior minimum-spacing-floor [edge_case]
  "Spacing base cannot be reduced below a minimum threshold"

  given
    A current DesignSystem with spacing base 2

  when interpret change "tighter layout"

  then returns change_result
    assert new spacing base >= 2


behavior multiple-changes-in-one-description [edge_case]
  "When a change description contains multiple instructions, the first recognized one is applied"

  given
    A current DesignSystem

  when interpret change "set primary-color to #1e40af and use Inter"

  then returns change_result
    assert status == "applied"
    assert at least one change is applied


depends on design-generation >= 1.0.0
