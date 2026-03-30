spec design-preview v1.0.0
title "Design Preview"

description
  Serves the current DesignSystem as a JSON API endpoint and broadcasts
  state changes over WebSocket for live rendering. The preview renders
  a full-screen app prototype per archetype — a realistic application
  page styled with the design tokens, not a documentation page of
  swatches and type scales. The prototype uses spec metadata as real
  content: spec names become navigation items, behavior counts become
  metric card values, spec categories become section headings. The
  preview occupies the full screen below the header with a white
  background. A detail panel slides in from the side when the user
  clicks elements, showing design system token details.

motivation
  The preview is the primary feedback surface for design iteration.
  Users need to see a realistic application — not a token catalog —
  to make meaningful design decisions. Rendering spec metadata as
  real content makes the prototype immediately recognizable as their
  project. Live WebSocket updates ensure the user sees changes
  instantly after each refine_layout call without page reloads.
  The JSON API enables programmatic access to the full design state
  for tooling integration.

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
    assert response contains layout configuration
    assert response contains component styles


behavior api-includes-spec-metadata [happy_path]
  "The design API response includes spec metadata for preview content"

  given
    A design state with spec metadata has been generated

  when GET /api/design

  then returns json_response
    assert response contains spec metadata
    assert spec metadata includes project name
    assert spec metadata includes spec categories
    assert spec metadata includes total behavior count


behavior api-returns-layout-config [happy_path]
  "The design API response includes structured layout configuration"

  given
    A design state with dashboard archetype has been generated

  when GET /api/design

  then returns json_response
    assert response contains layout
    assert layout contains sidebar configuration
    assert layout contains header configuration
    assert layout contains content sections


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


# Prototype rendering — archetype layouts

behavior render-dashboard-prototype [happy_path]
  "Dashboard archetype renders a full-screen app with sidebar, header, metric cards, and data table"

  given
    A design state with archetype "dashboard"
    Spec metadata with project name, spec names, and behavior counts

  when the React preview renders

  then
    assert the page occupies full screen below the header
    assert a sidebar is visible with navigation items from spec names
    assert a header bar shows the project name
    assert metric cards display real numbers from spec metadata
    assert a data table shows spec entries with names and versions


behavior render-ecommerce-prototype [happy_path]
  "E-commerce archetype renders a product grid with navigation header"

  given
    A design state with archetype "e-commerce"

  when the React preview renders

  then
    assert the page occupies full screen below the header
    assert a navigation header is visible
    assert a product grid with card components is visible


behavior render-content-prototype [happy_path]
  "Content archetype renders a reading-width article layout"

  given
    A design state with archetype "content"

  when the React preview renders

  then
    assert the page occupies full screen below the header
    assert content is constrained to a reading-comfortable width
    assert typography uses the design system type scale


behavior render-form-prototype [happy_path]
  "Form-heavy archetype renders a wizard with progress indicator"

  given
    A design state with archetype "form-heavy"

  when the React preview renders

  then
    assert the page occupies full screen below the header
    assert a progress indicator is visible
    assert form fields are visible


behavior render-generic-prototype [happy_path]
  "Generic archetype renders an inferred layout from spec structure"

  given
    A design state with archetype "generic"
    Content sections derived from spec analysis

  when the React preview renders

  then
    assert the page occupies full screen below the header
    assert content sections are rendered based on layout configuration


# Token application

behavior prototype-styled-with-tokens [happy_path]
  "All prototype elements are styled using design system tokens"

  given
    A design state with palette, type scale, and spacing

  when the React preview renders

  then
    assert colors in the rendered page match the palette tokens
    assert font sizes follow the type scale
    assert spacing between elements follows the spacing grid


behavior prototype-updates-live [happy_path]
  "The prototype re-renders when tokens change via WebSocket without page reload"

  given
    The preview is displaying a dashboard prototype
    A WebSocket connection is active

  when a design-update message arrives with a new primary color

  then
    assert the prototype re-renders with the new color
    assert no full page reload occurs


# Spec metadata as content

behavior metric-cards-show-real-data [happy_path]
  "Metric cards display actual spec count, behavior count, and entity count"

  given
    A dashboard prototype with spec metadata:
    29 specs, 653 behaviors, 14 entities

  when the React preview renders metric cards

  then
    assert a metric card shows 29 for spec count
    assert a metric card shows 653 for behavior count
    assert a metric card shows 14 for entity count


behavior nav-items-from-spec-names [happy_path]
  "Sidebar navigation items are derived from spec names"

  given
    A dashboard prototype with spec names:
    validate-command, watch-command, format-command, graph-command

  when the React preview renders the sidebar

  then
    assert navigation items include entries derived from spec names


behavior data-table-from-spec-list [happy_path]
  "Data table rows show actual spec names, versions, and behavior counts"

  given
    A dashboard prototype with spec metadata containing
    validate-command v2.1.0 with 30 behaviors

  when the React preview renders the data table

  then
    assert a row shows validate-command
    assert the row shows version 2.1.0
    assert the row shows behavior count 30


# Detail panel

behavior detail-panel-on-click [happy_path]
  "Clicking a design element opens a side panel with token details"

  given
    The preview is displaying a prototype

  when the user clicks on a styled element

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


behavior handle-missing-layout-config [error_case]
  "Fall back to a default layout when the design state has no layout configuration"

  given
    A design state exists but the layout configuration is missing

  when the React preview renders

  then
    assert a default single-column layout is used
    assert the prototype still renders with available tokens


# Edge cases

behavior empty-spec-metadata-fallback [edge_case]
  "Use placeholder content when spec metadata is empty"

  given
    A design state with no spec metadata

  when the React preview renders

  then
    assert navigation items use generic fallback labels
    assert metric cards show placeholder values
    assert the prototype is still visually complete


behavior white-background-full-screen [edge_case]
  "The preview page uses a white background and occupies the full viewport below the header"

  given
    A design state exists

  when the React preview renders

  then
    assert the page background is white
    assert the prototype occupies the full width
    assert the prototype starts immediately below the site header


depends on design-generation >= 1.0.0
