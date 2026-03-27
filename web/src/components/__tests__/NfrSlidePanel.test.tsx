import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, it, expect, vi } from "vitest"
import { NfrSlidePanel } from "../NfrSlidePanel"
import { mockNfr } from "@/test/fixtures"

describe("NfrSlidePanel", () => {
  /// nfr-panel-opens-on-click: Clicking an NFR card opens a slide panel with constraint details
  describe("nfr-panel-opens-on-click", () => {
    it("renders panel container but not content when nfr is null", () => {
      const { container } = render(
        <NfrSlidePanel nfr={null} isOpen={false} onClose={vi.fn()} />
      )
      const panel = container.querySelector("[data-testid='nfr-slide-panel']")
      expect(panel).not.toBeNull()
      expect(panel!.className).toContain("translate-x-full")
      expect(screen.queryByText(/v\d/)).not.toBeInTheDocument()
    })

    it("renders NFR category and version when open", () => {
      const nfr = mockNfr({ category: "performance", version: "1.1.0" })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("performance")).toBeInTheDocument()
      expect(screen.getByText(/v1\.1\.0/)).toBeInTheDocument()
    })

    it("shows info button for description", () => {
      const nfr = mockNfr({ description: "Performance constraints" })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByRole("button", { name: /nfr info/i })).toBeInTheDocument()
    })

    it("has correct sliding animation classes when open", () => {
      const nfr = mockNfr()
      const { container } = render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      const panel = container.querySelector("[data-testid='nfr-slide-panel']")!
      expect(panel.className).toContain("translate-x-0")
      expect(panel.className).not.toContain("translate-x-full")
    })

    it("has translate-x-full class when closed", () => {
      const nfr = mockNfr()
      const { container } = render(
        <NfrSlidePanel nfr={nfr} isOpen={false} onClose={vi.fn()} />
      )
      const panel = container.querySelector("[data-testid='nfr-slide-panel']")!
      expect(panel.className).toContain("translate-x-full")
    })

    it("lists each constraint with name", () => {
      const nfr = mockNfr({
        constraints: [
          { name: "api-latency", description: "Latency limit", constraint_type: "metric", threshold: "< 500ms", rule_text: null, violation: "critical", overridable: true },
          { name: "throughput", description: "Throughput floor", constraint_type: "metric", threshold: ">= 1000rps", rule_text: null, violation: "warning", overridable: false },
        ],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("api-latency")).toBeInTheDocument()
      expect(screen.getByText("throughput")).toBeInTheDocument()
    })

    it("shows metric type badge for metric constraints", () => {
      const nfr = mockNfr({
        constraints: [
          { name: "api-latency", description: "Latency limit", constraint_type: "metric", threshold: "< 500ms", rule_text: null, violation: "critical", overridable: true },
        ],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      const badge = screen.getByText("metric")
      expect(badge.className).toContain("blue")
    })

    it("shows rule type badge for rule constraints", () => {
      const nfr = mockNfr({
        constraints: [
          { name: "no-blocking", description: "No blocking calls", constraint_type: "rule", threshold: null, rule_text: "All I/O must be async", violation: "critical", overridable: false },
        ],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      const badge = screen.getByText("rule")
      expect(badge.className).toContain("purple")
    })

    it("shows threshold for metric constraints", () => {
      const nfr = mockNfr({
        constraints: [
          { name: "api-latency", description: "Latency limit", constraint_type: "metric", threshold: "< 500ms", rule_text: null, violation: "critical", overridable: true },
        ],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText(/< 500ms/)).toBeInTheDocument()
    })

    it("shows rule text for rule constraints", () => {
      const nfr = mockNfr({
        constraints: [
          { name: "no-blocking", description: "No blocking", constraint_type: "rule", threshold: null, rule_text: "All I/O must be async", violation: "critical", overridable: false },
        ],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText(/All I\/O must be async/)).toBeInTheDocument()
    })

    it("shows violation level", () => {
      const nfr = mockNfr({
        constraints: [
          { name: "api-latency", description: "Latency", constraint_type: "metric", threshold: "< 500ms", rule_text: null, violation: "critical", overridable: true },
        ],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText(/critical/)).toBeInTheDocument()
    })

    it("shows overridable status as yes when overridable", () => {
      const nfr = mockNfr({
        constraints: [
          { name: "api-latency", description: "Latency", constraint_type: "metric", threshold: "< 500ms", rule_text: null, violation: "critical", overridable: true },
        ],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("yes")).toBeInTheDocument()
    })

    it("shows overridable status as no when not overridable", () => {
      const nfr = mockNfr({
        constraints: [
          { name: "throughput", description: "Throughput", constraint_type: "metric", threshold: ">= 1000rps", rule_text: null, violation: "warning", overridable: false },
        ],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("no")).toBeInTheDocument()
    })

    it("shows referencing specs", () => {
      const nfr = mockNfr({
        referenced_by: ["auth-command", "validate-command"],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("auth-command")).toBeInTheDocument()
      expect(screen.getByText("validate-command")).toBeInTheDocument()
    })
  })

  /// nfr-panel-shows-referencing-specs: NFR panel lists which specs reference this NFR category
  describe("nfr-panel-shows-referencing-specs", () => {
    it("shows 'Referenced By' section header", () => {
      const nfr = mockNfr({
        referenced_by: ["auth-command", "validate-command"],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText(/Referenced By/i)).toBeInTheDocument()
    })

    it("lists all referencing spec names", () => {
      const nfr = mockNfr({
        referenced_by: ["auth-command", "validate-command"],
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("auth-command")).toBeInTheDocument()
      expect(screen.getByText("validate-command")).toBeInTheDocument()
    })

    it("does not show Referenced By section when no specs reference", () => {
      const nfr = mockNfr({ referenced_by: [] })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.queryByText(/Referenced By/i)).not.toBeInTheDocument()
    })
  })

  /// close behavior
  describe("close behavior", () => {
    it("calls onClose when close button is clicked", async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()
      const nfr = mockNfr()
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={onClose} />
      )
      const closeButton = screen.getByRole("button", { name: /close/i })
      await user.click(closeButton)
      expect(onClose).toHaveBeenCalled()
    })

    it("calls onClose when backdrop overlay is clicked", async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()
      const nfr = mockNfr()
      const { container } = render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={onClose} />
      )
      const overlay = container.querySelector("[data-testid='nfr-slide-panel-overlay']")!
      await user.click(overlay)
      expect(onClose).toHaveBeenCalled()
    })

    it("calls onClose when Escape key is pressed", async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()
      const nfr = mockNfr()
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={onClose} />
      )
      await user.keyboard("{Escape}")
      expect(onClose).toHaveBeenCalled()
    })
  })

  /// validation errors for invalid NFR
  describe("invalid NFR errors", () => {
    it("shows validation errors for invalid NFR", () => {
      const nfr = mockNfr({
        validation_status: { Invalid: ["parse error at line 3", "missing category"] },
        constraints: [],
        constraint_count: 0,
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("parse error at line 3")).toBeInTheDocument()
      expect(screen.getByText("missing category")).toBeInTheDocument()
    })

    it("shows red error icon in header for invalid NFR", () => {
      const nfr = mockNfr({
        validation_status: { Invalid: ["parse error"] },
        constraints: [],
        constraint_count: 0,
      })
      render(
        <NfrSlidePanel nfr={nfr} isOpen={true} onClose={vi.fn()} />
      )
      const header = screen.getByText("performance").closest("div")!
      const parent = header.parentElement!
      const svgs = parent.querySelectorAll("svg")
      const redIcon = Array.from(svgs).find(
        (svg) => svg.classList.contains("text-red-400")
      )
      expect(redIcon).toBeTruthy()
    })
  })
})
