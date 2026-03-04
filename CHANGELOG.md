# Changelog

All notable changes to minter will be documented in this file.
## [1.0.1] - 2026-03-04

### Fixed

- **cli:** List available topics when minter guide is run without arguments

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


