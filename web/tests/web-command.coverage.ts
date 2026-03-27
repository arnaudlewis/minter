// Coverage mapping for web-command.spec behaviors
// Each @minter tag links a spec behavior to its test file(s)
//
// Frontend tests: web/src/components/__tests__/*.test.tsx (vitest)
// Backend tests: tests/web_command.rs (cargo test)

// --- Header ---
// @minter:unit web-command/header-displays-metrics
// @minter:unit web-command/lock-regeneration-feedback
// @minter:unit web-command/lock-drift-tooltip
// @minter:unit web-command/header-shows-invalid-tags-badge
// @minter:unit web-command/reconnect-on-disconnect
// Tested in: web/src/components/__tests__/MetricsBar.test.tsx

// --- Spec cards ---
// @minter:unit web-command/spec-card-displays-summary
// @minter:unit web-command/spec-card-status-green
// @minter:unit web-command/spec-card-status-warning
// @minter:unit web-command/spec-card-status-error
// @minter:unit web-command/spec-card-shows-dep-errors
// @minter:unit web-command/filter-specs-by-search
// @minter:unit web-command/broken-spec-stays-visible
// Tested in: web/src/components/__tests__/SpecCardGrid.test.tsx

// --- Slide panel ---
// @minter:unit web-command/panel-opens-on-card-click
// @minter:unit web-command/panel-shows-behaviors
// @minter:unit web-command/panel-shows-nfr-refs
// @minter:unit web-command/panel-shows-dependencies
// @minter:unit web-command/panel-shows-errors
// @minter:unit web-command/panel-search-behaviors
// @minter:unit web-command/panel-behavior-detail
// Tested in: web/src/components/__tests__/SpecSlidePanel.test.tsx

// --- Backend / integration ---
// @minter:e2e web-command/launch-starts-server
// @minter:e2e web-command/server-port-fallback
// @minter:e2e web-command/live-update-on-file-change
// Tested in: tests/web_command.rs
