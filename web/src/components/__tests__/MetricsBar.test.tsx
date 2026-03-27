import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, it, expect, vi } from "vitest"
import { MetricsBar } from "../MetricsBar"
import { mockState } from "@/test/fixtures"

describe("MetricsBar", () => {
  /// header-displays-metrics: Header shows project-level metrics
  describe("header-displays-metrics", () => {
    it("renders spec count at readable size", () => {
      const state = mockState({
        specs: Array.from({ length: 20 }, (_, i) => ({
          name: `spec-${i}`,
          version: "1.0.0",
          path: `/specs/spec-${i}.spec`,
          behavior_count: 25,
          behaviors: Array.from({ length: 25 }, (_, j) => ({
            name: `b-${j}`,
            covered: j < 24,
            test_types: j < 24 ? ["unit"] : [],
            category: "happy_path",
            nfr_refs: [],
            description: "",
          })),
          validation_status: "Valid" as const,
        })),
      })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText(/20/)).toBeInTheDocument()
      expect(screen.getByText(/specs/i)).toBeInTheDocument()
    })

    it("renders behavior count", () => {
      const state = mockState({
        specs: [
          {
            name: "s1",
            version: "1.0.0",
            path: "/s1.spec",
            behavior_count: 499,
            behaviors: [],
            validation_status: "Valid",
          },
        ],
      })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText(/499/)).toBeInTheDocument()
      expect(screen.getByText(/behaviors/i)).toBeInTheDocument()
    })

    it("renders NFR count", () => {
      const state = mockState({ nfr_count: 22 })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText(/22/)).toBeInTheDocument()
      expect(screen.getByText(/NFRs/)).toBeInTheDocument()
    })

    it("renders tag count", () => {
      const state = mockState({ test_count: 145 })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText(/145/)).toBeInTheDocument()
      expect(screen.getByText(/tags/i)).toBeInTheDocument()
    })

    it("renders coverage percentage with colored bar", () => {
      const state = mockState({ coverage_covered: 490, coverage_total: 499 })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText(/98%/)).toBeInTheDocument()
      expect(screen.getByText(/Coverage/i)).toBeInTheDocument()
    })

    it("colors coverage bar green when >= 80%", () => {
      const state = mockState({ coverage_covered: 90, coverage_total: 100 })
      const { container } = render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      const bar = container.querySelector("[style*='width: 90%']")
      expect(bar).toBeInTheDocument()
      expect(bar?.className).toContain("emerald")
    })

    it("colors coverage bar amber when >= 50% and < 80%", () => {
      const state = mockState({ coverage_covered: 60, coverage_total: 100 })
      const { container } = render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      const bar = container.querySelector("[style*='width: 60%']")
      expect(bar).toBeInTheDocument()
      expect(bar?.className).toContain("amber")
    })

    it("colors coverage bar red when < 50%", () => {
      const state = mockState({ coverage_covered: 30, coverage_total: 100 })
      const { container } = render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      const bar = container.querySelector("[style*='width: 30%']")
      expect(bar).toBeInTheDocument()
      expect(bar?.className).toContain("red")
    })
  })

  /// header-shows-lock-status: Header shows lock status with regenerate button
  describe("header-shows-lock-status", () => {
    it("shows lock status text as aligned", () => {
      const state = mockState({
        integrity: {
          specs: "Aligned",
          nfrs: "Aligned",
          tests: "Aligned",
          lock_status: "Aligned",
        },
      })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText(/aligned/i)).toBeInTheDocument()
    })

    it("shows lock status text as drifted", () => {
      const state = mockState({
        integrity: {
          specs: "Aligned",
          nfrs: "Aligned",
          tests: "Aligned",
          lock_status: "Drifted",
        },
      })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText(/drifted/i)).toBeInTheDocument()
    })

    it("shows Regenerate button when lock is drifted", () => {
      const state = mockState({
        integrity: {
          specs: "Aligned",
          nfrs: "Aligned",
          tests: "Aligned",
          lock_status: "Drifted",
        },
      })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByRole("button", { name: /regenerate/i })).toBeInTheDocument()
    })

    it("hides Regenerate button when lock is aligned", () => {
      const state = mockState({
        integrity: {
          specs: "Aligned",
          nfrs: "Aligned",
          tests: "Aligned",
          lock_status: "Aligned",
        },
      })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.queryByRole("button", { name: /regenerate/i })).not.toBeInTheDocument()
    })

    it("calls onRegenerateLock when Regenerate is clicked", async () => {
      const user = userEvent.setup()
      const onRegenerateLock = vi.fn()
      const state = mockState({
        integrity: {
          specs: "Aligned",
          nfrs: "Aligned",
          tests: "Aligned",
          lock_status: "Drifted",
        },
      })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={onRegenerateLock}
        />
      )
      await user.click(screen.getByRole("button", { name: /regenerate/i }))
      expect(onRegenerateLock).toHaveBeenCalled()
    })

    it("shows loading spinner on Regenerate button when lockLoading is true", () => {
      const state = mockState({
        integrity: {
          specs: "Aligned",
          nfrs: "Aligned",
          tests: "Aligned",
          lock_status: "Drifted",
        },
      })
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={true}
          onRegenerateLock={vi.fn()}
        />
      )
      const button = screen.getByRole("button", { name: /regenerat/i })
      expect(button).toBeDisabled()
      // Spinner should be rendered (an svg with animate-spin class)
      const spinner = button.querySelector(".animate-spin")
      expect(spinner).toBeInTheDocument()
    })
  })

  /// reconnect-on-disconnect: connection status indicator
  describe("header-shows-connection-status", () => {
    it("shows green indicator when connected", () => {
      const state = mockState()
      const { container } = render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      const dot = container.querySelector(".bg-emerald-400")
      expect(dot).toBeInTheDocument()
    })

    it("shows red indicator when disconnected", () => {
      const state = mockState()
      const { container } = render(
        <MetricsBar
          state={state}
          connected={false}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      const dot = container.querySelector(".bg-red-400")
      expect(dot).toBeInTheDocument()
    })

    it('shows "connected" text when connected', () => {
      const state = mockState()
      render(
        <MetricsBar
          state={state}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText("connected")).toBeInTheDocument()
    })

    it('shows "disconnected" text when disconnected', () => {
      const state = mockState()
      render(
        <MetricsBar
          state={state}
          connected={false}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      expect(screen.getByText("disconnected")).toBeInTheDocument()
    })
  })

  describe("loading state", () => {
    it("shows loading skeleton when loading", () => {
      const { container } = render(
        <MetricsBar
          state={null}
          connected={false}
          loading={true}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      const skeleton = container.querySelector(".animate-pulse")
      expect(skeleton).toBeInTheDocument()
    })
  })

  /// header-shows-invalid-tags-badge
  describe("header-shows-invalid-tags-badge", () => {
    it("shows red badge with count when there are invalid tags", () => {
      render(
        <MetricsBar
          state={mockState({ invalid_tags: [
            { file: "tests/a.rs", line: 1, message: "unknown" },
            { file: "tests/b.rs", line: 2, message: "unknown" },
            { file: "tests/c.rs", line: 3, message: "unknown" },
          ] })}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
          invalidTagCount={3}
          onShowInvalidTags={vi.fn()}
        />
      )
      expect(screen.getByText("3")).toBeInTheDocument()
    })

    it("hides badge when there are no invalid tags", () => {
      render(
        <MetricsBar
          state={mockState()}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
          invalidTagCount={0}
          onShowInvalidTags={vi.fn()}
        />
      )
      expect(screen.queryByRole("button", { name: /invalid/i })).not.toBeInTheDocument()
    })

    it("calls onShowInvalidTags when badge is clicked", async () => {
      const user = userEvent.setup()
      const onShow = vi.fn()
      render(
        <MetricsBar
          state={mockState({ invalid_tags: [
            { file: "tests/a.rs", line: 1, message: "unknown behavior" },
          ] })}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
          invalidTagCount={1}
          onShowInvalidTags={onShow}
        />
      )
      await user.click(screen.getByText("1"))
      expect(onShow).toHaveBeenCalled()
    })
  })

  /// lock-regeneration-feedback
  describe("lock-regeneration-feedback", () => {
    it("shows success indicator after lock regeneration", () => {
      render(
        <MetricsBar
          state={mockState()}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
          lockSuccess={true}
        />
      )
      expect(screen.getByText(/regenerated/i)).toBeInTheDocument()
    })
  })

  /// lock-drift-tooltip
  describe("lock-drift-tooltip", () => {
    it("shows drift reasons in tooltip when drifted", async () => {
      const user = userEvent.setup()
      render(
        <MetricsBar
          state={mockState({
            integrity: { specs: "Drifted", nfrs: "Aligned", tests: "Aligned", lock_status: "Drifted" },
            drift: {
              modified_specs: ["specs/auth.spec"],
              unlocked_specs: ["specs/new-feature.spec"],
              modified_nfrs: [], unlocked_nfrs: [],
              modified_tests: [], missing_tests: [],
            },
          })}
          connected={true}
          loading={false}
          lockLoading={false}
          onRegenerateLock={vi.fn()}
        />
      )
      const driftedText = screen.getByText("drifted")
      await user.hover(driftedText)
      expect(screen.getByText(/auth\.spec/)).toBeInTheDocument()
      expect(screen.getByText(/new-feature\.spec/)).toBeInTheDocument()
    })
  })
})
