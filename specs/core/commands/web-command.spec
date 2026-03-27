spec web-command v4.0.0
title "Web Dashboard Command"

description
  The minter web command launches a local web dashboard (axum + React)
  that provides a real-time view of an entire minter project. Specs
  are displayed as compact cards in a grid. Clicking a card opens a
  slide-over panel from the right (Notion-style) showing full spec
  details: behaviors, NFR references, dependencies, errors. Items
  within the panel are clickable for deeper details. WebSocket
  provides live updates. The only action is Lock regeneration.

motivation
  The terminal UI has limitations with scrolling, color rendering, and
  screen real estate. A web dashboard removes these constraints and
  provides a richer, more accessible interface for spec authors during
  active development.

nfr
  performance#watch-revalidation-latency
  reliability#no-silent-data-loss
  operability#zero-config


# Startup

behavior launch-starts-server [happy_path]
  "Start the web server, open the browser, and serve the dashboard"

  given
    A project with specs/ directory containing valid .spec files
    Port 4321 is available

  when minter web

  then emits stdout
    assert output contains "4321"

  then
    assert the default browser is opened to http://localhost:4321
    assert the server serves a single-page application


behavior server-port-fallback [error_case]
  "Try port+1 if the default port is already in use"

  given
    Port 4321 is already in use by another process

  when minter web

  then
    assert the server starts on port 4322


# Header

behavior header-displays-metrics [happy_path]
  "Header shows project metrics and lock status in a compact single bar"

  given
    A project with 20 specs, 499 behaviors, 22 NFR constraints, 145 tags
    Coverage is 98%
    Lock is aligned

  when the dashboard loads

  then
    assert header shows spec count, behavior count, NFR count, tag count
    assert header shows coverage percentage with colored progress bar
    assert header shows lock status
    assert header shows WebSocket connection indicator


behavior lock-regeneration-feedback [happy_path]
  "Lock regenerate button provides clear visual feedback"

  given
    Lock status is drifted and Regenerate button is visible

  when user clicks Regenerate

  then
    assert button shows a loading spinner during regeneration
    assert lock status updates to aligned after completion
    assert a brief success indicator appears for 2 seconds


behavior header-shows-invalid-tags-badge [happy_path]
  "Header shows a red error badge when there are invalid @minter tags"

  given
    Test files contain 3 @minter tags referencing non-existent behaviors

  when the dashboard renders

  then
    assert header shows a red badge with the count 3
    assert clicking the badge opens a panel listing the invalid tags
    assert each invalid tag shows file path, line number, and error message
    assert the badge is hidden when there are no invalid tags


# Spec cards

behavior spec-card-displays-summary [happy_path]
  "Each card shows spec name, version, behavior count, and coverage bar"

  given
    auth-command v1.2.0 has 12 behaviors, 10 covered, 2 uncovered

  when the dashboard renders

  then
    assert auth-command card shows spec name and version
    assert auth-command card shows behavior count
    assert auth-command card shows coverage mini-bar


behavior spec-card-status-green [happy_path]
  "Valid spec with 100% coverage shows green status"

  given
    auth-command is valid and all behaviors are covered

  when the dashboard renders

  then
    assert auth-command card shows a green check indicator


behavior spec-card-status-warning [happy_path]
  "Valid spec with uncovered behaviors shows warning status"

  given
    auth-command is valid but 2 behaviors are uncovered

  when the dashboard renders

  then
    assert auth-command card shows an amber warning indicator
    assert auth-command card lists uncovered behavior names


behavior spec-card-status-error [error_case]
  "Invalid spec shows error status with messages on the card"

  given
    scaffold-command has a parse error

  when the dashboard renders

  then
    assert scaffold-command card shows a red error indicator
    assert scaffold-command card shows the error message
    assert scaffold-command card shows 0 behaviors


behavior spec-card-shows-dep-errors [happy_path]
  "Card shows dependency errors inline"

  given
    auth-command depends on user-command which does not exist

  when the dashboard renders

  then
    assert auth-command card shows the dependency error


# Slide-over panel (Notion-style)

behavior panel-opens-on-card-click [happy_path]
  "Clicking a card opens a slide-over panel from the right"

  given
    The dashboard shows spec cards

  when user clicks on auth-command card

  then
    assert a panel slides in from the right covering about 50% of the screen
    assert the cards remain visible behind the panel
    assert the panel shows auth-command name and version
    assert the panel can be closed with the X button or Escape key


behavior panel-shows-behaviors [happy_path]
  "Panel lists all behaviors with coverage status and test types"

  given
    auth-command has behaviors login, logout, refresh-token
    login is covered by unit and e2e tests
    refresh-token is uncovered

  when user opens the panel for auth-command

  then
    assert the panel lists all behaviors
    assert each behavior shows its description text inline
    assert login shows covered status with unit and e2e badges
    assert refresh-token shows uncovered status
    assert each behavior is clickable


behavior panel-shows-nfr-refs [happy_path]
  "Panel shows NFR references at spec level and behavior level"

  given
    auth-command has spec-level NFR refs to performance and reliability
    login behavior has a behavior-level NFR ref to performance

  when user opens the panel for auth-command

  then
    assert the panel shows spec-level NFR references
    assert behavior-level NFR references are shown alongside each behavior


behavior panel-shows-dependencies [happy_path]
  "Panel shows spec dependencies"

  given
    auth-command depends on user-command >= 1.0.0

  when user opens the panel for auth-command

  then
    assert the panel shows the dependency list


behavior panel-shows-errors [error_case]
  "Panel shows validation errors for invalid specs"

  given
    scaffold-command has parse errors

  when user opens the panel for scaffold-command

  then
    assert the panel shows validation error messages


behavior panel-search-behaviors [happy_path]
  "Panel has a search input to filter behaviors"

  given
    auth-command has 12 behaviors and the panel is open

  when user types "login" in the panel search input

  then
    assert only behaviors matching login are shown
    assert clearing the search restores all behaviors


behavior panel-behavior-detail [happy_path]
  "Clicking a behavior shows its details in a popover"

  given
    The panel is open for auth-command
    login behavior is covered by unit and e2e tests

  when user clicks on login behavior

  then
    assert a detail view shows the behavior name
    assert the detail view shows which tests cover this behavior
    assert the detail view shows the behavior category tag


# Search

behavior filter-specs-by-search [happy_path]
  "Search bar filters spec cards in real-time"

  given
    The dashboard has specs: auth, billing, payments, user-profile

  when user types "bill" in the search bar

  then
    assert only billing card is visible
    assert clearing the search restores all cards


# Real-time

behavior live-update-on-file-change [happy_path]
  "Dashboard updates automatically when files change on disk"

  given
    The dashboard shows auth-command with 4 behaviors

  when auth-command.spec is modified to add a fifth behavior

  then
    assert auth-command card updates
    assert header metrics refresh
    assert no page reload is needed


behavior reconnect-on-disconnect [happy_path]
  "Auto-reconnect WebSocket with visual indicator"

  given
    The dashboard is connected via WebSocket

  when the connection drops

  then
    assert header shows disconnected indicator
    assert the dashboard reconnects automatically


# Error handling

behavior broken-spec-stays-visible [error_case]
  "Specs with parse errors remain as cards with error indicator"

  given
    auth-command is showing as valid

  when auth-command is modified to contain invalid syntax

  then
    assert auth-command card remains visible with red error indicator
    assert auth-command card shows 0 behaviors


depends on config >= 1.0.0
depends on validate-command >= 2.1.0
depends on coverage-command >= 1.4.0
depends on lock-command >= 1.0.0
