spec design-preview v2.0.0
title "Design Preview"

description
  Serves the current DesignSystem as a JSON API endpoint and broadcasts
  state changes over WebSocket for live rendering. The preview renders
  a component showcase page — a library of UI components all styled
  with the generated design tokens, similar to shadcn/ui's component
  library. The page structure is always the same regardless of
  archetype: navigation, cards, data table, buttons, form elements,
  alerts, typography, color palette, and spacing visualization. Spec
  metadata populates components with real content: spec names become
  navigation items, behavior counts become metric card values, spec
  entries populate the data table. The archetype influences token
  defaults (colors, spacing) but not page structure.

motivation
  A component showcase always looks good because it renders shadcn-
  quality components styled with the project's tokens. Users see
  every component in their design system at once, making it easy to
  evaluate whether the palette, typography, and spacing work across
  different contexts. The previous archetype-specific templates were
  fragile — each template needed custom maintenance, the generic
  fallback looked mediocre, and adding new archetypes meant adding
  new templates. With the showcase approach, one page structure
  works for every project. Users customize through iteration, not
  template selection. Live WebSocket updates ensure changes appear
  instantly after each refine_layout call without page reloads.

nfr
  operability#deterministic-output
  performance#watch-revalidation-latency
  reliability#no-silent-data-loss


# JSON API

behavior api-returns-design-system [happy_path]
  "GET /api/design returns the full DesignSystem as JSON"

  given
    A design state has been generated and persisted

  when GET /api/design

  then returns json_response
    assert status code == 200
    assert response contains archetype
    assert response contains palette
    assert response contains type scale
    assert response contains spacing scale
    assert response contains component styles


behavior api-includes-spec-metadata [happy_path]
  "The design API response includes spec metadata for showcase content"

  given
    A design state with spec metadata has been generated

  when GET /api/design

  then returns json_response
    assert response contains spec metadata
    assert spec metadata includes project name
    assert spec metadata includes spec categories
    assert spec metadata includes total behavior count


behavior api-not-found-without-state [error_case]
  "Return 404 when no design state has been generated yet"

  given
    No design state exists at .minter/design-state.json

  when GET /api/design

  then returns json_response
    assert status code == 404
    assert response contains guidance on generating a design first


# WebSocket updates

behavior websocket-broadcasts-on-change [happy_path]
  "Broadcast a design-update message when the design state changes"

  given
    A WebSocket client is connected
    A design state exists

  when the design state is updated by a refine_layout call

  then emits websocket_message
    assert message type == "design-update"
    assert message contains the updated DesignSystem


behavior websocket-sends-initial-state [happy_path]
  "Send the current design state when a WebSocket client connects"

  given
    A design state exists

  when a new WebSocket client connects

  then emits websocket_message
    assert message contains the current DesignSystem


behavior websocket-handles-no-state [edge_case]
  "Send a no-design message when a client connects with no design state"

  given
    No design state exists

  when a new WebSocket client connects

  then emits websocket_message
    assert message indicates no design state is available


# Component showcase rendering

behavior render-navigation-component [happy_path]
  "The showcase renders a navigation component styled with tokens and populated with spec names"

  given
    A design state with palette and spec metadata
    Spec metadata includes spec names:
    validate-command, watch-command, format-command, graph-command

  when the component showcase renders

  then
    assert a navigation component is visible
    assert navigation items are derived from spec names
    assert navigation uses palette colors for background and text
    assert navigation has an active state indicator on one item


behavior render-metric-cards [happy_path]
  "The showcase renders metric cards with real project numbers from spec metadata"

  given
    A design state with spec metadata:
    29 specs, 653 behaviors, 14 entities

  when the component showcase renders

  then
    assert metric cards are visible
    assert a card shows 29 for spec count
    assert a card shows 653 for behavior count
    assert a card shows 14 for entity count
    assert cards are styled with palette colors and component border radius


behavior render-data-table [happy_path]
  "The showcase renders a data table with real spec entries from spec metadata"

  given
    A design state with spec metadata containing
    validate-command v2.1.0 with 30 behaviors

  when the component showcase renders

  then
    assert a data table is visible
    assert a row shows validate-command
    assert the row shows version 2.1.0
    assert the row shows behavior count 30
    assert the table uses palette colors for header and row styling


behavior render-button-variants [happy_path]
  "The showcase renders primary, secondary, outline, ghost, and destructive button styles"

  given
    A design state with palette and component styles

  when the component showcase renders

  then
    assert a primary button is visible using the primary color
    assert a secondary button is visible using the secondary color
    assert an outline button is visible with a border and no fill
    assert a ghost button is visible with no border and no fill
    assert a destructive button is visible using the error semantic color


behavior render-form-elements [happy_path]
  "The showcase renders form inputs, selects, checkboxes, and radios styled with tokens"

  given
    A design state with palette and component styles

  when the component showcase renders

  then
    assert text inputs are visible with border styling from tokens
    assert a select dropdown is visible
    assert checkboxes are visible
    assert radio buttons are visible
    assert form elements use the type scale for label and input text


behavior render-alerts-badges [happy_path]
  "The showcase renders success, warning, error, and info alerts and badges using semantic colors"

  given
    A design state with semantic colors for success, warning, error, and info

  when the component showcase renders

  then
    assert a success alert is visible using the success semantic color
    assert a warning alert is visible using the warning semantic color
    assert an error alert is visible using the error semantic color
    assert an info alert is visible using the info semantic color
    assert status badges are visible using semantic colors


behavior render-typography-showcase [happy_path]
  "The showcase renders each type scale level at its actual size"

  given
    A design state with a 7-level type scale

  when the component showcase renders

  then
    assert each type scale level is rendered with a label and sample text
    assert font sizes visually increase from caption to display
    assert the font family matches the design system type scale


behavior render-color-palette [happy_path]
  "The showcase renders color swatches for all palette roles and shade ramps"

  given
    A design state with primary, secondary, accent, and neutral colors
    Each color has a shade ramp

  when the component showcase renders

  then
    assert primary color swatches are visible with shade variants
    assert secondary color swatches are visible
    assert accent color swatches are visible
    assert semantic color swatches are visible for success, warning, and error
    assert each swatch displays its hex value


behavior render-spacing-visualization [happy_path]
  "The showcase renders visual spacing steps from the spacing grid"

  given
    A design state with a spacing grid of 8 steps

  when the component showcase renders

  then
    assert spacing steps are visualized with labeled blocks
    assert the smallest step matches the spacing base
    assert steps visually increase in size
    assert step labels show the pixel values


# View layout

behavior design-view-split-layout [happy_path]
  "Design view renders as a split layout when the chat panel is available"

  given
    Server state has has_claude == true
    A design state exists

  when the Design view renders

  then
    assert the preview occupies the right portion of the viewport
    assert the left portion is reserved for the chat panel
    assert the preview still renders the full showcase


behavior design-view-full-width [happy_path]
  "Design view renders the preview at full width when no chat is available"

  given
    Server state has has_claude == false

  when the Design view renders

  then
    assert the preview occupies the full viewport width below the header
    assert a message suggests installing Claude Code for chat-based design iteration


# Token application

behavior showcase-styled-with-tokens [happy_path]
  "All showcase components are styled using design system tokens"

  given
    A design state with palette, type scale, and spacing

  when the component showcase renders

  then
    assert colors in the rendered page match the palette tokens
    assert font sizes follow the type scale
    assert spacing between elements follows the spacing grid


behavior showcase-updates-live [happy_path]
  "The showcase re-renders when tokens change via WebSocket without page reload"

  given
    The preview is displaying the component showcase
    A WebSocket connection is active

  when a design-update message arrives with a new primary color

  then
    assert the showcase re-renders with the new color
    assert no full page reload occurs


# Detail panel

behavior detail-panel-on-click [happy_path]
  "Clicking a showcase component opens a side panel with token details"

  given
    The preview is displaying the component showcase

  when the user clicks on a styled component

  then
    assert a detail panel slides in from the side
    assert the panel shows the relevant design token values
    assert the panel can be closed


# Error cases

behavior handle-corrupted-state [error_case]
  "Display an error message when the design state file is corrupted"

  given
    The design state file exists but contains invalid JSON

  when the React preview attempts to render

  then
    assert an error message is displayed to the user
    assert the error message suggests regenerating the design


behavior handle-missing-tokens [error_case]
  "Render available components when some token categories are missing"

  given
    A design state exists but the spacing scale is missing

  when the component showcase renders

  then
    assert components that do not require spacing still render
    assert a fallback spacing value is used for affected components


# Edge cases

behavior empty-spec-metadata-fallback [edge_case]
  "Use placeholder content when spec metadata is empty"

  given
    A design state with no spec metadata

  when the component showcase renders

  then
    assert navigation items use generic fallback labels
    assert metric cards show placeholder values
    assert the data table shows placeholder rows
    assert the showcase is still visually complete


behavior white-background-full-screen [edge_case]
  "The preview page uses a white background and occupies the full viewport below the header"

  given
    A design state exists

  when the component showcase renders

  then
    assert the page background is white
    assert the showcase occupies the full width
    assert the showcase starts immediately below the site header


depends on design-generation >= 2.0.0
