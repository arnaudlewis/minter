spec ui-command v1.6.0
title "UI Command"

description
  The minter ui command launches an interactive terminal dashboard
  (ratatui + crossterm) that provides a reactive, single-screen view
  of an entire minter project. It combines spec inventory, behavior
  coverage, lock integrity status, and project actions into a unified
  TUI. The command is gated behind the ui Cargo feature flag
  (--features ui). It reuses the existing notify file watcher with
  300ms debounce for reactive updates when specs, NFRs, or test files
  change on disk. The UI supports both keyboard and mouse interaction.
  Every CLI action is available from the TUI — actions are context-aware,
  targeting the selected spec when one is highlighted or operating
  globally when no spec is selected.

motivation
  Today, understanding a minter project requires running multiple
  commands: validate, coverage, lock, graph, inspect, format,
  scaffold, guide. Each gives a partial view. The TUI consolidates all
  project intelligence into one screen with live updates, reducing
  context-switching and making the full project state visible at a
  glance. It is the primary interface for spec authors during active
  development.

nfr
  performance#watch-revalidation-latency
  reliability#no-silent-data-loss
  operability#zero-config


# Startup and overview — happy path

behavior launch-displays-overview [happy_path]
  "Launch the TUI and display the overview bar with correct project counts"

  given
    A project with 5 .spec files containing 18 behaviors total
    2 .nfr files containing 5 constraints total
    3 @minter tags across test files
    A minter.lock file exists and all hashes match current state
    Test coverage is 14/18 (77%)

  when minter ui

  then
    assert overview bar displays spec count, behavior count, nfr constraint count, test tag count, coverage percentage, and lock status
    assert overview bar shows 5 specs, 18 behaviors, 5 nfrs, 3 tests, 77%, and aligned


behavior launch-displays-specs-list [happy_path]
  "Display the specs list panel with all specs, versions, and behavior counts"

  given
    specs/auth.spec v1.0.0 has 4 behaviors
    specs/billing.spec v2.1.0 has 6 behaviors

  when minter ui

  then
    assert specs panel lists auth v1.0.0 with 4 behaviors
    assert specs panel lists billing v2.1.0 with 6 behaviors
    assert specs are listed in alphabetical order


behavior launch-displays-integrity-aligned [happy_path]
  "Display integrity panel showing all-aligned status when lock matches"

  given
    minter.lock exists and was generated from the current state
    All spec, NFR, and test file hashes match the lock
    No new files exist outside the lock
    No locked files are missing from disk

  when minter ui

  then
    assert integrity panel shows specs as aligned
    assert integrity panel shows nfrs as aligned
    assert integrity panel shows tests as aligned


behavior launch-displays-integrity-drifted [happy_path]
  "Display integrity panel showing drift details when lock mismatches"

  given
    minter.lock exists
    specs/auth.spec has been modified since lock (hash mismatch)
    specs/new-feature.spec exists but is not in the lock
    tests/removed_test.rs is in the lock but deleted from disk

  when minter ui

  then
    assert integrity panel shows specs as drifted
    assert integrity panel lists auth.spec as modified
    assert integrity panel lists new-feature.spec as unlocked
    assert integrity panel lists removed_test.rs as missing


# Spec navigation — happy path

behavior navigate-specs-keyboard [happy_path]
  "Navigate the specs list with keyboard arrows and expand with enter"

  given
    The TUI is running with specs/auth.spec and specs/billing.spec loaded
    The specs list panel is focused

  when user navigates to billing.spec with arrow keys and presses enter

  then
    assert billing.spec row is highlighted as selected
    assert billing.spec behaviors are visible below the spec row
    assert each behavior shows its coverage status


behavior navigate-specs-mouse [happy_path]
  "Navigate the specs list with mouse click to expand and collapse"

  given
    The TUI is running with multiple specs loaded
    The specs list panel is visible

  when user clicks on a spec row to toggle expand state

  then
    assert clicking a collapsed spec expands it to show behaviors
    assert clicking an expanded spec collapses it to hide behaviors
    assert mouse scroll navigates the specs list


behavior expand-spec-shows-coverage [happy_path]
  "Expanded spec shows each behavior with its test coverage details"

  given
    specs/auth.spec has behaviors: login, logout, refresh-token
    login is covered by unit and e2e tests
    logout is covered by e2e test only
    refresh-token is uncovered

  when user expands auth.spec in the specs list

  then
    assert login shows covered status with unit and e2e test types
    assert logout shows covered status with e2e test type
    assert refresh-token shows uncovered status


behavior selection-highlight-visible [happy_path]
  "The selected spec row has a clearly visible highlight"

  given
    The TUI is running with multiple specs loaded
    The specs list panel is focused

  when user navigates to a spec row with arrow keys

  then
    assert the selected row has a distinct cyan or blue background highlight
    assert the highlight is clearly distinguishable from unselected rows
    assert the highlight moves as the user navigates up and down


# Context-aware actions — targeted (spec selected)

behavior validate-targeted-spec [happy_path]
  "Validate only the selected spec when a spec is highlighted and validate is triggered"

  given
    The TUI is running with specs/auth.spec and specs/billing.spec loaded
    auth.spec is selected in the specs list
    auth.spec has a validation error

  when user presses v to trigger validate

  then
    assert results area shows validation output for auth.spec only
    assert results area does not show billing.spec output
    assert results area includes the validation error message for auth


behavior deep-validate-targeted-spec [happy_path]
  "Deep validate only the selected spec when a spec is highlighted and deep validate is triggered"

  given
    The TUI is running with specs/auth.spec loaded
    auth.spec is selected in the specs list
    auth.spec depends on user >= 1.0.0
    specs/user.spec exists and is valid

  when user presses d to trigger deep validate

  then
    assert results area shows validation output for auth.spec with dependency resolution
    assert results area includes user.spec in the dependency tree
    assert the deep flag is applied to the targeted spec


behavior coverage-targeted-spec [happy_path]
  "Show coverage for only the selected spec when a spec is highlighted and coverage is triggered"

  given
    The TUI is running with specs/auth.spec and specs/billing.spec loaded
    auth.spec is selected in the specs list
    auth.spec has 3 behaviors with 2 covered

  when user presses c to trigger coverage

  then
    assert results area shows coverage output for auth.spec only
    assert results area shows 2/3 behaviors covered for auth
    assert results area does not show billing.spec coverage


behavior graph-targeted-spec [happy_path]
  "Show dependency graph for the selected spec when a spec is highlighted and graph is triggered"

  given
    The TUI is running with specs/auth.spec loaded
    auth.spec is selected in the specs list
    auth.spec depends on user >= 1.0.0

  when user presses g to trigger graph

  then
    assert results area shows dependency graph for auth.spec
    assert results area shows auth with an edge to user
    assert results area does not show the full project graph


behavior inspect-targeted-spec [happy_path]
  "Show metadata for the selected spec when a spec is highlighted and inspect is triggered"

  given
    The TUI is running with specs/auth.spec loaded
    auth.spec is selected in the specs list
    auth.spec has 4 behaviors: 2 happy_path, 1 error_case, 1 edge_case
    auth.spec depends on user >= 1.0.0

  when user presses n to trigger inspect

  then
    assert results area shows inspect output for auth.spec
    assert results area shows behavior count and category distribution
    assert results area shows dependency list


# Context-aware actions — global (no spec selected)

behavior validate-global-no-selection [happy_path]
  "Validate all specs in the project when no spec is selected and validate is triggered"

  given
    The TUI is running with specs/auth.spec and specs/billing.spec loaded
    No spec is selected in the specs list
    auth.spec has a validation error

  when user presses v to trigger validate

  then
    assert results area shows validation output for all specs
    assert results area shows auth with a failure indicator
    assert results area shows billing with a success indicator


behavior coverage-global-no-selection [happy_path]
  "Show full project coverage when no spec is selected and coverage is triggered"

  given
    The TUI is running with specs/auth.spec and specs/billing.spec loaded
    No spec is selected in the specs list
    Total coverage is 8/10 (80%)

  when user presses c to trigger coverage

  then
    assert results area shows coverage output for all specs
    assert results area shows 8/10 and 80 percent total


behavior graph-global-no-selection [happy_path]
  "Show the full dependency graph when no spec is selected and graph is triggered"

  given
    The TUI is running with specs/a.spec and specs/b.spec loaded
    No spec is selected in the specs list
    a.spec depends on b >= 1.0.0

  when user presses g to trigger graph

  then
    assert results area shows the full project dependency graph
    assert results area shows a with an edge to b


# Always-global actions

behavior trigger-lock-action [happy_path]
  "Trigger the lock generation action globally regardless of spec selection"

  given
    The TUI is running
    The integrity panel shows drifted status
    auth.spec is selected in the specs list

  when user triggers the lock action

  then
    assert results area confirms lock file was generated for the entire project
    assert integrity panel refreshes to show aligned status
    assert lock covers all specs, not just the selected one


behavior lock-action-refreshes-integrity [happy_path]
  "Integrity panel updates immediately after the lock action generates a new lock file"

  given
    The TUI is running with integrity showing drifted status
    specs have been modified since the last lock

  when user triggers the lock action

  then
    assert the lock file is regenerated
    assert the integrity panel refreshes to show aligned status
    assert the refresh happens automatically after the action completes


# New actions — happy path

behavior trigger-format-reference [happy_path]
  "Display the DSL grammar reference in the results panel when format is triggered"

  given
    The TUI is running

  when user presses f to trigger format reference

  then
    assert results area shows the DSL grammar reference
    assert results area includes spec grammar sections: spec, title, description, behavior, given, when, then
    assert results area is scrollable for the full grammar content


behavior trigger-scaffold-spec [happy_path]
  "Display a spec skeleton in the results panel when scaffold is triggered"

  given
    The TUI is running

  when user presses s to trigger scaffold

  then
    assert results area shows the scaffold output
    assert results area includes skeleton with spec, title, description, behavior sections
    assert the scaffold output could be copied to create a new spec file


behavior trigger-guide-topics [happy_path]
  "Display the development guide topic list or content when guide is triggered"

  given
    The TUI is running

  when user presses ? to trigger guide

  then
    assert results area shows the available guide topics
    assert results area includes methodology, workflow, authoring, smells, nfr, context, coverage


# Action bar — happy path

behavior action-bar-with-selection [happy_path]
  "Display context-aware action labels when a spec is selected"

  given
    The TUI is running with multiple specs loaded
    auth.spec is selected in the specs list

  when the action bar renders

  then
    assert action bar shows v:validate label indicating targeted validation
    assert action bar shows d:deep-validate label
    assert action bar shows c:coverage label indicating targeted coverage
    assert action bar shows g:graph label indicating targeted graph
    assert action bar shows n:inspect label
    assert action bar shows f:format label
    assert action bar shows s:scaffold label
    assert action bar shows ?:guide label
    assert action bar shows lock label as always-global


behavior action-bar-without-selection [happy_path]
  "Display global action labels when no spec is selected"

  given
    The TUI is running with multiple specs loaded
    No spec is selected in the specs list

  when the action bar renders

  then
    assert action bar shows v:validate label indicating global validation
    assert action bar shows c:coverage label indicating global coverage
    assert action bar shows g:graph label indicating full graph
    assert action bar shows f:format label
    assert action bar shows s:scaffold label
    assert action bar shows ?:guide label
    assert action bar shows lock label
    assert action bar does not show n:inspect label since inspect requires a selected spec
    assert action bar does not show d:deep-validate label since deep validate requires a selected spec


# Validation status — happy path

behavior validation-status-icons [happy_path]
  "Each spec shows a live validation status icon in the specs list"

  given
    The TUI is running with specs/auth.spec (valid) and specs/billing.spec (valid)

  when the specs list renders

  then
    assert each spec row shows a green checkmark when the spec is valid
    assert each spec row shows a red cross when the spec has parse or semantic errors
    assert the icons update automatically when specs change on disk


behavior broken-spec-stays-visible [happy_path]
  "A spec with parse errors remains visible in the specs list with error status"

  given
    The TUI is running with specs/auth.spec loaded and valid
    The file watcher is monitoring the specs directory

  when specs/auth.spec is modified to contain invalid syntax

  then
    assert specs list still shows auth.spec
    assert auth.spec shows a red cross validation status icon
    assert auth.spec shows 0 behaviors since it cannot be parsed
    assert the spec does not disappear from the list


behavior integrity-shows-validation-errors [happy_path]
  "Integrity panel displays all validation errors when specs have parse or semantic errors"

  given
    The TUI is running with specs/auth.spec (broken syntax) and specs/billing.spec (valid)
    No spec is selected in the specs list

  when the integrity panel renders

  then
    assert integrity panel shows a validation errors section
    assert the section header includes the count of invalid specs
    assert auth.spec is listed with a red cross and its error messages
    assert billing.spec is not listed in the validation errors section
    assert each spec shows up to 3 error messages with a truncation indicator for more


behavior esc-clears-all [happy_path]
  "Pressing Esc clears both the action result and spec selection in one press"

  given
    The TUI is running with auth.spec selected
    The results panel shows a previous validate action output

  when user presses Esc

  then
    assert the results panel is replaced by the integrity panel
    assert no spec is selected in the specs list
    assert the action bar updates to show global mode


behavior action-bar-shows-esc-hint [happy_path]
  "Action bar shows an Esc hint when a spec is selected"

  given
    The TUI is running with auth.spec selected in the specs list

  when the action bar renders

  then
    assert action bar shows an Esc label indicating return to global mode
    assert the Esc label is visually distinct from other action labels


behavior integrity-shows-uncovered-behaviors [happy_path]
  "Integrity panel lists all uncovered behaviors grouped by spec when coverage is below 100%"

  given
    The TUI is running with specs/auth.spec (3 behaviors) and specs/billing.spec (2 behaviors)
    auth/login and auth/logout are covered by tests
    auth/refresh-token, billing/create-invoice, and billing/cancel-invoice are uncovered
    No spec is selected in the specs list

  when the integrity panel renders

  then
    assert integrity panel shows an uncovered behaviors section
    assert the section lists auth/refresh-token under auth
    assert the section lists create-invoice and cancel-invoice under billing
    assert each uncovered behavior is displayed with its spec name for context


behavior integrity-shows-invalid-tags [happy_path]
  "Integrity panel lists invalid @minter tags that reference non-existent behaviors"

  given
    The TUI is running with specs/auth.spec containing behavior login
    Test file contains @minter:unit login (valid) and @minter:unit nonexistent (invalid)

  when the integrity panel renders

  then
    assert integrity panel shows an invalid tags section
    assert the section lists the file and line with the invalid tag
    assert valid tags are not listed in the invalid tags section


behavior integrity-shows-dependency-errors [happy_path]
  "Integrity panel shows dependency resolution errors when specs have broken dependencies"

  given
    The TUI is running with specs/auth.spec depending on user >= 1.0.0
    specs/user.spec does not exist

  when the integrity panel renders

  then
    assert integrity panel shows a dependency errors section
    assert the section lists the unresolved dependency
    assert the dependency error includes the spec name and the missing dependency


# Reactive updates — happy path

behavior reactive-update-spec-change [happy_path]
  "Reactively update the UI when a spec file changes on disk"

  given
    The TUI is running with specs/auth.spec showing 4 behaviors
    The file watcher is monitoring the specs directory

  when specs/auth.spec is modified to add a fifth behavior

  then
    assert overview bar updates the behavior count
    assert specs list updates auth.spec to show 5 behaviors
    assert the update happens automatically without manual user action


behavior reactive-update-test-change [happy_path]
  "Reactively update coverage when a test file changes on disk"

  given
    The TUI is running with 8/10 behaviors covered
    The file watcher is monitoring the test directories

  when a test file is modified to add a @minter tag covering an uncovered behavior

  then
    assert overview bar updates the coverage percentage
    assert the newly covered behavior updates its status in the expanded spec
    assert the update happens automatically without manual user action


# Quit — happy path

behavior quit-with-q [happy_path]
  "Quit the TUI cleanly when the user presses q"

  given
    The TUI is running

  when user presses q

  then emits process_exit
    assert code == 0

  then
    assert terminal is restored to its original state


behavior quit-with-ctrl-c [happy_path]
  "Quit the TUI cleanly when the user presses Ctrl+C"

  given
    The TUI is running

  when user presses Ctrl+C

  then emits process_exit
    assert code == 0

  then
    assert terminal is restored to its original state


# Error cases

behavior launch-without-specs-directory [error_case]
  "Display an error in the UI when no specs directory is found"

  given
    No specs/ directory exists
    No minter.config.json exists

  when minter ui

  then
    assert the UI renders with an error message about missing specs directory
    assert the actions panel is still accessible for the user


behavior launch-without-lock-file [error_case]
  "Show no-lock status and make lock action available when no lock file exists"

  given
    specs/ directory exists with valid specs
    No minter.lock file exists

  when minter ui

  then
    assert overview bar shows no lock for lock status
    assert integrity panel shows no lock file message
    assert the lock action is available and enabled


behavior action-fails-display-error [error_case]
  "Display the error in the results panel when a triggered action fails"

  given
    The TUI is running
    specs/broken.spec has a parse error

  when user triggers the validate action

  then
    assert results area shows the error output for broken.spec
    assert the UI remains interactive after the error is displayed


behavior inspect-requires-selection [error_case]
  "Inspect action is unavailable when no spec is selected"

  given
    The TUI is running
    No spec is selected in the specs list

  when user presses n to trigger inspect

  then
    assert the action is ignored or a message indicates that a spec must be selected
    assert the UI remains interactive


behavior deep-validate-requires-selection [error_case]
  "Deep validate action is unavailable when no spec is selected"

  given
    The TUI is running
    No spec is selected in the specs list

  when user presses d to trigger deep validate

  then
    assert the action is ignored or a message indicates that a spec must be selected
    assert the UI remains interactive


# Edge cases

behavior large-project-scrollable [edge_case]
  "Support scrollable specs list for projects with many specs"

  given
    A project with 50 spec files
    The terminal window can display 20 rows in the specs panel

  when user navigates down past the visible area

  then
    assert specs list scrolls to keep the selected spec visible
    assert the viewport follows the selection as the user navigates
    assert scrolling up brings earlier specs back into view


behavior integrity-panel-scrollable [edge_case]
  "Support scrollable integrity panel when content overflows"

  given
    The TUI is running with many uncovered behaviors and validation errors
    The integrity panel content exceeds the visible area

  when user presses Page Down or Page Up

  then
    assert the integrity panel scrolls to reveal hidden content
    assert Page Down scrolls one page forward
    assert Page Up scrolls one page backward


behavior zero-coverage-no-crash [edge_case]
  "Display 0% coverage without crash when no tests exist"

  given
    specs/auth.spec has 4 behaviors
    No test files contain @minter tags

  when minter ui

  then
    assert overview bar shows 0 percent coverage
    assert specs panel shows all behaviors as uncovered
    assert the UI remains stable and interactive


behavior lock-outdated-shows-drift [edge_case]
  "Show specific drift details when lock file exists but is outdated"

  given
    minter.lock was generated from a previous project state
    3 spec files have been modified since lock
    1 new NFR file was added since lock
    2 test files were deleted since lock

  when minter ui

  then
    assert integrity panel shows 3 specs modified
    assert integrity panel shows 1 nfr unlocked
    assert integrity panel shows 2 tests missing
    assert each drifted file is listed by name


behavior action-bar-updates-on-selection-change [edge_case]
  "Action bar updates dynamically as the user selects and deselects specs"

  given
    The TUI is running with multiple specs loaded
    No spec is initially selected

  when user selects a spec then deselects it

  then
    assert action bar shows inspect and deep-validate labels while a spec is selected
    assert action bar hides inspect and deep-validate labels when no spec is selected
    assert the transition is immediate with no flicker


depends on config >= 1.0.0
depends on validate-command >= 2.1.0
depends on coverage-command >= 1.4.0
depends on lock-command >= 1.0.0
depends on graph-command >= 1.4.0
depends on inspect-command >= 1.1.0
depends on format-command >= 1.2.0
depends on scaffold-command >= 1.2.0
depends on guide-command >= 1.2.0
