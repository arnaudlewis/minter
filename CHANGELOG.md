# Changelog

All notable changes to minter will be documented in this file.
## [Unreleased]

### Added

- **graph:** Behavior-level change detection in validate responses.
  The graph cache (now schema v4) tracks individual behaviors with SHA-256
  hashes and maintains a baseline snapshot. When `validate` is called via
  MCP and behaviors have been added, removed, or modified since the last
  validation, the response includes a `changes` section with the full diff.
  This enables agents to detect orphaned tests and dead code after spec
  iterations.
- **graph:** `VersionChange` tracking — when a spec's version changes between
  validations, the `changes` section includes `version_change` with `from`
  and `to` values.
- **guide:** New `refinement` topic (`minter guide refinement`) — documents the
  spec iteration cleanup workflow: how to act on removed/modified behaviors,
  trace dead code, and update stale tests.
- **content:** `initialize_minter` and `guide workflow` updated with a 6th
  principle ("Specs evolve") and an expanded Phase 5 covering cleanup after
  spec changes.

### Changed

- **graph:** Cache schema upgraded from v3 to v4. Existing caches are rebuilt
  automatically on first use. No manual action required.

### Important: Spec Agent Update Required

The shipped agent prompt (`docs/spec-agent.md`) has been updated. If you
maintain a copy of this prompt in your own agent configuration, you must
update it to include:

- **Mode 4 (Refinement):** Now includes a "Handling validate changes"
  subsection that instructs the agent to act on `changes` in validate
  responses — identifying orphaned tests for removed behaviors and stale
  tests for modified behaviors.
- **Core Loop:** Adds a refinement path:
  `validate → act on changes → cleanup tests/code → validate`
- **Test Guidance:** Adds cleanup instructions for removed/modified behaviors.

Without this update, your agent will receive the change detection data but
won't know to act on it.

## [2.0.0] - 2026-03-28

### Added

- **parser:** Accept any @minter tag type
- **cli:** Add config, lock, and ci commands
- **parser:** Reject trailing content and fix orphaned tags
- **cli:** Add web dashboard with real-time spec monitoring
- **mcp:** Refactor MCP as spec authoring assistant, remove web feature flag

### CI/CD

- Upgrade GitHub Actions to Node.js 24 compatible versions

### Documentation

- Add squash merge rule to CLAUDE.md
- Restructure documentation as wiki

### Maintenance

- Add frontend test script, deduplicate helpers, improve docs navigation

## [1.1.1] - 2026-03-24

### Fixed

- Trigger first open source release

### Maintenance

- Prepare for open source release
- Release v1.1.1

## [1.1.0] - 2026-03-09

### Added

- **mcp:** Expose coverage tool

### Maintenance

- Release v1.1.0

## [1.0.1] - 2026-03-04

### Fixed

- **cli:** List available topics when minter guide is run without arguments

### Maintenance

- Release v1.0.1

## [1.0.0] - 2026-03-04

### Added

- **cli:** Add anti-contamination guidance to smells, authoring, and NFR guides
- **cli:** Add coverage tagging guide topic
- **cli:** Add coverage-command spec v1.1.0
- **cli:** Implement minter coverage command
- **cli:** Add coverage command routing to CLI spec
- **cli:** Compact coverage display with --verbose flag
- **cli:** Add coverage tagging directive to Phase 3 guides

### Changed

- **cli:** Replace explain command with guide command
- **validator:** Extract shared orchestration to core

### Documentation

- Move guide-command.spec to core/commands/
- Add coverage command and update guide references
- Update coverage command docs with compact display
- Fix coverage output example to match actual display

### Fixed

- **mcp:** Add coverage topic to guide tool description

### Maintenance

- Release v1.0.0

### Testing

- **validator:** Add unit tests for inspect, graph BFS, and cache skip logic
- Add @minter coverage tags to all test and bench files

## [0.1.0] - 2026-02-27

### Added

- Minter v0.1.0 — spec compiler and validator for structured behavioral specifications

### CI/CD

- Use public changelog config for release notes
- Show both public and internal changelog in release preview
- Use prebuilt git-cliff binary in tag job
- Use git-cliff action for changelog generation in tag job
- Fix cross-compilation by installing cross via cargo
- Run build in dry run mode to catch failures early
- Show dry run vs execute in workflow run name
- Fix x86_64 macOS build — use macos-latest (cross-compile)

### Maintenance

- Release v0.1.0
- Release v0.1.0


