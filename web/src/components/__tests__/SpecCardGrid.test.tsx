import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, it, expect, vi } from "vitest"
import { SpecCardGrid } from "../SpecCardGrid"
import { mockSpec } from "@/test/fixtures"

describe("SpecCardGrid", () => {
  /// spec-card-displays-summary: Each spec card shows name, version, behavior count, and coverage
  describe("spec-card-displays-summary", () => {
    it("shows spec name on the card", () => {
      const spec = mockSpec({ name: "auth-command", version: "1.2.0" })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      expect(screen.getByText("auth-command")).toBeInTheDocument()
    })

    it("shows spec version on the card", () => {
      const spec = mockSpec({ name: "auth-command", version: "1.2.0" })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      expect(screen.getByText("v1.2.0")).toBeInTheDocument()
    })

    it("shows behavior count on the card", () => {
      const spec = mockSpec({
        name: "auth-command",
        behavior_count: 12,
        behaviors: Array.from({ length: 12 }, (_, i) => ({
          name: `b-${i}`,
          covered: i < 10,
          test_types: i < 10 ? ["unit"] : [],
          category: "happy_path",
          nfr_refs: [],
        })),
      })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      expect(screen.getByText(/12 behaviors/)).toBeInTheDocument()
    })

    it("shows coverage percentage as mini-bar on the card", () => {
      const spec = mockSpec({
        name: "auth-command",
        behavior_count: 12,
        behaviors: Array.from({ length: 12 }, (_, i) => ({
          name: `b-${i}`,
          covered: i < 10,
          test_types: i < 10 ? ["unit"] : [],
          category: "happy_path",
          nfr_refs: [],
        })),
      })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      // Coverage appears in both the text summary and the mini-bar
      const matches = screen.getAllByText(/83%/)
      expect(matches.length).toBeGreaterThanOrEqual(1)
    })

    it("shows spec description on the card", () => {
      const spec = mockSpec({
        name: "auth-command",
        description: "Handles user authentication flows",
      })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      expect(screen.getByText("Handles user authentication flows")).toBeInTheDocument()
    })

    it("shows NFR category badges on the card", () => {
      const spec = mockSpec({
        name: "auth-command",
        nfr_refs: ["performance#api-latency", "performance#throughput", "reliability#no-data-loss"],
      })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      // Should show unique categories: performance, reliability
      expect(screen.getByText("performance")).toBeInTheDocument()
      expect(screen.getByText("reliability")).toBeInTheDocument()
    })

    it("hides NFR badges when no NFR refs", () => {
      const spec = mockSpec({
        name: "auth-command",
        nfr_refs: [],
      })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      expect(screen.queryByText("performance")).not.toBeInTheDocument()
      expect(screen.queryByText("reliability")).not.toBeInTheDocument()
    })
  })

  /// spec-card-valid-fully-covered: A valid spec with 100% coverage shows green status
  describe("spec-card-valid-fully-covered", () => {
    it("shows green check indicator for fully covered valid spec", () => {
      const spec = mockSpec({
        name: "auth-command",
        validation_status: "Valid",
        behavior_count: 12,
        behaviors: Array.from({ length: 12 }, (_, i) => ({
          name: `b-${i}`,
          covered: true,
          test_types: ["unit"],
          category: "happy_path",
          nfr_refs: [],
        })),
      })
      const { container } = render(
        <SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />
      )
      const card = screen.getByText("auth-command").closest("[data-testid='spec-card']")!
      const svgs = card.querySelectorAll("svg")
      const greenIcon = Array.from(svgs).find(
        (svg) => svg.classList.contains("text-emerald-400")
      )
      expect(greenIcon).toBeTruthy()
    })
  })

  /// spec-card-valid-partially-covered: A valid spec with uncovered behaviors shows warning
  describe("spec-card-valid-partially-covered", () => {
    it("shows amber warning indicator for partially covered spec", () => {
      const spec = mockSpec({
        name: "auth-command",
        validation_status: "Valid",
        behavior_count: 12,
        behaviors: Array.from({ length: 12 }, (_, i) => ({
          name: `b-${i}`,
          covered: i < 10,
          test_types: i < 10 ? ["unit"] : [],
          category: "happy_path",
          nfr_refs: [],
        })),
      })
      const { container } = render(
        <SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />
      )
      const card = screen.getByText("auth-command").closest("[data-testid='spec-card']")!
      const svgs = card.querySelectorAll("svg")
      const amberIcon = Array.from(svgs).find(
        (svg) => svg.classList.contains("text-amber-400")
      )
      expect(amberIcon).toBeTruthy()
    })

    it("lists uncovered behavior names on the card", () => {
      const spec = mockSpec({
        name: "auth-command",
        validation_status: "Valid",
        behavior_count: 3,
        behaviors: [
          { name: "login", covered: true, test_types: ["unit"], category: "happy_path", nfr_refs: [], description: "" },
          { name: "logout", covered: false, test_types: [], category: "happy_path", nfr_refs: [], description: "" },
          { name: "refresh-token", covered: false, test_types: [], category: "error_case", nfr_refs: [] },
        ],
      })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      expect(screen.getByText(/2 uncovered behaviors/i)).toBeInTheDocument()
    })
  })

  /// spec-card-invalid: A spec with parse errors shows error status with messages
  describe("spec-card-invalid", () => {
    it("shows red error indicator for invalid spec", () => {
      const spec = mockSpec({
        name: "scaffold-command",
        validation_status: { Invalid: ["line 5: Expected 'motivation'"] },
        behaviors: [],
        behavior_count: 0,
      })
      const { container } = render(
        <SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />
      )
      const card = screen.getByText("scaffold-command").closest("[data-testid='spec-card']")!
      const svgs = card.querySelectorAll("svg")
      const redIcon = Array.from(svgs).find(
        (svg) => svg.classList.contains("text-red-400")
      )
      expect(redIcon).toBeTruthy()
    })

    it("shows error message on the card", () => {
      const spec = mockSpec({
        name: "scaffold-command",
        validation_status: { Invalid: ["line 5: Expected 'motivation'"] },
        behaviors: [],
        behavior_count: 0,
      })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      expect(screen.getByText(/line 5: Expected 'motivation'/)).toBeInTheDocument()
    })

    it("shows 0 behaviors for invalid spec", () => {
      const spec = mockSpec({
        name: "scaffold-command",
        validation_status: { Invalid: ["parse error"] },
        behaviors: [],
        behavior_count: 0,
      })
      render(<SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />)
      expect(screen.getByText(/0 behaviors/)).toBeInTheDocument()
    })
  })

  /// spec-card-shows-dependency-errors: A spec card shows dependency errors
  describe("spec-card-shows-dependency-errors", () => {
    it("shows dependency error on the card", () => {
      const spec = mockSpec({
        name: "auth-command",
        version: "1.2.0",
      })
      render(
        <SpecCardGrid
          specs={[spec]}
          depErrors={["missing dep: user-command >= 1.0.0"]}
          onSelectSpec={vi.fn()}
        />
      )
      expect(screen.getByText(/missing dep: user-command >= 1.0.0/)).toBeInTheDocument()
    })
  })

  /// filter-specs-by-search: Search bar filters spec cards in real-time
  describe("filter-specs-by-search", () => {
    it("filters specs by name when user types in search", async () => {
      const user = userEvent.setup()
      const specs = [
        mockSpec({ name: "auth" }),
        mockSpec({ name: "billing" }),
        mockSpec({ name: "payments" }),
        mockSpec({ name: "user-profile" }),
      ]
      render(
        <SpecCardGrid specs={specs} depErrors={[]} onSelectSpec={vi.fn()} />
      )

      expect(screen.getByText("auth")).toBeInTheDocument()
      expect(screen.getByText("billing")).toBeInTheDocument()
      expect(screen.getByText("payments")).toBeInTheDocument()
      expect(screen.getByText("user-profile")).toBeInTheDocument()

      const searchInput = screen.getByPlaceholderText(/search/i)
      await user.type(searchInput, "bill")

      expect(screen.getByText("billing")).toBeInTheDocument()
      expect(screen.queryByText("auth")).not.toBeInTheDocument()
      expect(screen.queryByText("payments")).not.toBeInTheDocument()
      expect(screen.queryByText("user-profile")).not.toBeInTheDocument()
    })

    it("restores all specs when search is cleared", async () => {
      const user = userEvent.setup()
      const specs = [
        mockSpec({ name: "auth" }),
        mockSpec({ name: "billing" }),
      ]
      render(
        <SpecCardGrid specs={specs} depErrors={[]} onSelectSpec={vi.fn()} />
      )

      const searchInput = screen.getByPlaceholderText(/search/i)
      await user.type(searchInput, "bill")
      expect(screen.queryByText("auth")).not.toBeInTheDocument()

      await user.clear(searchInput)
      expect(screen.getByText("auth")).toBeInTheDocument()
      expect(screen.getByText("billing")).toBeInTheDocument()
    })
  })

  /// broken-spec-stays-visible: Specs with parse errors remain visible with error indicator
  describe("broken-spec-stays-visible", () => {
    it("shows broken spec with error icon and 0 behaviors", () => {
      const spec = mockSpec({
        name: "auth",
        validation_status: { Invalid: ["unexpected token at line 12"] },
        behaviors: [],
        behavior_count: 0,
      })
      render(
        <SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={vi.fn()} />
      )
      expect(screen.getByText("auth")).toBeInTheDocument()
      expect(screen.getByText(/0 behaviors/)).toBeInTheDocument()
      const card = screen.getByText("auth").closest("[data-testid='spec-card']")!
      const svgs = card.querySelectorAll("svg")
      const redIcon = Array.from(svgs).find(
        (svg) => svg.classList.contains("text-red-400")
      )
      expect(redIcon).toBeTruthy()
    })
  })

  /// click on card calls onSelectSpec
  describe("card click", () => {
    it("calls onSelectSpec when card is clicked", async () => {
      const user = userEvent.setup()
      const onSelectSpec = vi.fn()
      const spec = mockSpec({ name: "auth" })
      render(
        <SpecCardGrid specs={[spec]} depErrors={[]} onSelectSpec={onSelectSpec} />
      )
      await user.click(screen.getByText("auth"))
      expect(onSelectSpec).toHaveBeenCalledWith(spec)
    })
  })
})
