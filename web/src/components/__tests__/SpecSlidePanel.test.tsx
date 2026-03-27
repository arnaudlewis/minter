import { render, screen, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, it, expect, vi } from "vitest"
import { SpecSlidePanel } from "../SpecSlidePanel"
import { mockSpec } from "@/test/fixtures"

describe("SpecSlidePanel", () => {
  /// panel-opens-on-card-click: Clicking a card opens a slide-over panel from the right
  describe("panel-opens-on-card-click", () => {
    it("renders panel container but not content when spec is null", () => {
      const { container } = render(
        <SpecSlidePanel spec={null} isOpen={false} onClose={vi.fn()} />
      )
      // Panel container should always be in the DOM for CSS transitions
      const panel = container.querySelector("[data-testid='slide-panel']")
      expect(panel).not.toBeNull()
      expect(panel!.className).toContain("translate-x-full")
      // But no spec content should be rendered inside
      expect(screen.queryByText(/v\d/)).not.toBeInTheDocument()
    })

    it("renders spec name and version when open", () => {
      const spec = mockSpec({ name: "auth-command", version: "1.2.0" })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("auth-command")).toBeInTheDocument()
      expect(screen.getByText(/v1\.2\.0/)).toBeInTheDocument()
    })

    it("has correct sliding animation classes when open", () => {
      const spec = mockSpec({ name: "auth-command" })
      const { container } = render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      const panel = container.querySelector("[data-testid='slide-panel']")!
      expect(panel.className).toContain("translate-x-0")
      expect(panel.className).not.toContain("translate-x-full")
    })

    it("has translate-x-full class when closed", () => {
      const spec = mockSpec({ name: "auth-command" })
      const { container } = render(
        <SpecSlidePanel spec={spec} isOpen={false} onClose={vi.fn()} />
      )
      const panel = container.querySelector("[data-testid='slide-panel']")!
      expect(panel.className).toContain("translate-x-full")
      expect(panel.className).not.toContain("translate-x-0")
    })

    it("panel has translate-x-full when spec is null and isOpen is false", () => {
      const { container } = render(
        <SpecSlidePanel spec={null} isOpen={false} onClose={vi.fn()} />
      )
      const panel = container.querySelector("[data-testid='slide-panel']")!
      expect(panel.className).toContain("translate-x-full")
    })

    it("calls onClose when close button is clicked", async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()
      const spec = mockSpec({ name: "auth-command" })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={onClose} />
      )
      const closeButton = screen.getByRole("button", { name: /close/i })
      await user.click(closeButton)
      expect(onClose).toHaveBeenCalled()
    })

    it("calls onClose when backdrop overlay is clicked", async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()
      const spec = mockSpec({ name: "auth-command" })
      const { container } = render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={onClose} />
      )
      const overlay = container.querySelector("[data-testid='slide-panel-overlay']")!
      await user.click(overlay)
      expect(onClose).toHaveBeenCalled()
    })

    it("calls onClose when Escape key is pressed", async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()
      const spec = mockSpec({ name: "auth-command" })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={onClose} />
      )
      await user.keyboard("{Escape}")
      expect(onClose).toHaveBeenCalled()
    })
  })

  /// panel-shows-behaviors: Panel lists all behaviors with coverage status and test types
  describe("panel-shows-behaviors", () => {
    it("lists all behaviors", () => {
      const spec = mockSpec({
        name: "auth-command",
        behaviors: [
          { name: "login", covered: true, test_types: ["unit", "e2e"], category: "happy_path", nfr_refs: [] },
          { name: "logout", covered: true, test_types: ["e2e"], category: "happy_path", nfr_refs: [] },
          { name: "refresh-token", covered: false, test_types: [], category: "error_case", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("login")).toBeInTheDocument()
      expect(screen.getByText("logout")).toBeInTheDocument()
      expect(screen.getByText("refresh-token")).toBeInTheDocument()
    })

    it("shows behavior description inline", () => {
      const spec = mockSpec({
        behaviors: [
          { name: "login", description: "Authenticate a user with credentials", covered: true, test_types: ["unit"], category: "happy_path", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("Authenticate a user with credentials")).toBeInTheDocument()
    })

    it("shows test type badges for covered behaviors", () => {
      const spec = mockSpec({
        behaviors: [
          { name: "login", covered: true, test_types: ["unit", "e2e"], category: "happy_path", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("unit")).toBeInTheDocument()
      expect(screen.getByText("e2e")).toBeInTheDocument()
    })

    it("shows colored badges for different test types", () => {
      const spec = mockSpec({
        behaviors: [
          {
            name: "login",
            covered: true,
            test_types: ["unit", "e2e", "integration", "benchmark"],
            category: "happy_path",
            nfr_refs: [],
          },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      const unitBadge = screen.getByText("unit")
      const e2eBadge = screen.getByText("e2e")
      const integrationBadge = screen.getByText("integration")
      const benchmarkBadge = screen.getByText("benchmark")

      expect(unitBadge.className).toContain("blue")
      expect(e2eBadge.className).toContain("purple")
      expect(integrationBadge.className).toContain("emerald")
      expect(benchmarkBadge.className).toContain("orange")
    })

    it("shows uncovered marker for uncovered behaviors", () => {
      const spec = mockSpec({
        behaviors: [
          { name: "refresh-token", covered: false, test_types: [], category: "error_case", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("uncovered")).toBeInTheDocument()
    })

    it("shows category tag for each behavior", () => {
      const spec = mockSpec({
        behaviors: [
          { name: "login", covered: true, test_types: ["unit"], category: "happy_path", nfr_refs: [] },
          { name: "error-handler", covered: false, test_types: [], category: "error_case", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("happy_path")).toBeInTheDocument()
      expect(screen.getByText("error_case")).toBeInTheDocument()
    })
  })

  /// panel-shows-nfr-refs: Panel shows NFR references at spec level and behavior level
  describe("panel-shows-nfr-refs", () => {
    it("displays spec-level NFR refs", () => {
      const spec = mockSpec({
        nfr_refs: ["performance#api-latency", "reliability#no-data-loss"],
        behaviors: [
          { name: "login", covered: true, test_types: ["unit"], category: "happy_path", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText(/performance#api-latency/)).toBeInTheDocument()
      expect(screen.getByText(/reliability#no-data-loss/)).toBeInTheDocument()
    })

    it("shows behavior-level NFR refs next to each behavior", () => {
      const spec = mockSpec({
        behaviors: [
          { name: "login", covered: true, test_types: ["unit"], category: "happy_path", nfr_refs: ["performance#api-latency"] },
          { name: "logout", covered: true, test_types: ["e2e"], category: "happy_path", nfr_refs: [] },
        ],
        nfr_refs: [],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      // The behavior-level NFR ref should appear alongside the login behavior
      const loginRow = screen.getByText("login").closest("[data-testid='behavior-row']")!
      expect(loginRow.textContent).toContain("performance#api-latency")
    })
  })

  /// panel-shows-dependencies: Panel shows spec dependencies
  describe("panel-shows-dependencies", () => {
    it("shows dependency list", () => {
      const spec = mockSpec({
        dependencies: ["user-command >= 1.0.0", "billing-api >= 2.3.0"],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("user-command >= 1.0.0")).toBeInTheDocument()
      expect(screen.getByText("billing-api >= 2.3.0")).toBeInTheDocument()
    })

    it("does not render dependencies section when empty", () => {
      const spec = mockSpec({
        dependencies: [],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.queryByText(/dependencies/i)).not.toBeInTheDocument()
    })
  })

  /// panel-shows-errors: Panel shows validation errors for invalid specs
  describe("panel-shows-errors", () => {
    it("shows validation error messages", () => {
      const spec = mockSpec({
        validation_status: {
          Invalid: ["parse error at line 5", "missing title"],
        },
        behaviors: [],
        behavior_count: 0,
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("parse error at line 5")).toBeInTheDocument()
      expect(screen.getByText("missing title")).toBeInTheDocument()
    })

    it("error messages are styled with red", () => {
      const spec = mockSpec({
        validation_status: {
          Invalid: ["parse error at line 5"],
        },
        behaviors: [],
        behavior_count: 0,
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      const errorText = screen.getByText("parse error at line 5")
      const errorContainer = errorText.closest("[data-testid='error-message']")!
      expect(errorContainer.className).toContain("red")
    })
  })

  /// panel-search-behaviors: Panel has a search input to filter behaviors
  describe("panel-search-behaviors", () => {
    it("filters behaviors by name when user types in search", async () => {
      const user = userEvent.setup()
      const spec = mockSpec({
        behavior_count: 3,
        behaviors: [
          { name: "login", covered: true, test_types: ["unit"], category: "happy_path", nfr_refs: [] },
          { name: "logout", covered: true, test_types: ["e2e"], category: "happy_path", nfr_refs: [] },
          { name: "refresh-token", covered: false, test_types: [], category: "error_case", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )

      const searchInput = screen.getByPlaceholderText(/search behaviors/i)
      await user.type(searchInput, "login")

      expect(screen.getByText("login")).toBeInTheDocument()
      expect(screen.queryByText("refresh-token")).not.toBeInTheDocument()
    })

    it("restores all behaviors when search is cleared", async () => {
      const user = userEvent.setup()
      const spec = mockSpec({
        behavior_count: 3,
        behaviors: [
          { name: "login", covered: true, test_types: ["unit"], category: "happy_path", nfr_refs: [] },
          { name: "logout", covered: true, test_types: ["e2e"], category: "happy_path", nfr_refs: [] },
          { name: "refresh-token", covered: false, test_types: [], category: "error_case", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )

      const searchInput = screen.getByPlaceholderText(/search behaviors/i)
      await user.type(searchInput, "login")
      expect(screen.queryByText("refresh-token")).not.toBeInTheDocument()

      await user.clear(searchInput)
      expect(screen.getByText("login")).toBeInTheDocument()
      expect(screen.getByText("logout")).toBeInTheDocument()
      expect(screen.getByText("refresh-token")).toBeInTheDocument()
    })
  })

  /// panel-behavior-detail: Behavior details always visible
  describe("panel-behavior-detail", () => {
    it("shows test types, category, and covered badge without clicking", () => {
      const spec = mockSpec({
        behaviors: [
          { name: "login", description: "Auth", covered: true, test_types: ["unit", "e2e"], category: "happy_path", nfr_refs: ["performance#api-latency"] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("covered")).toBeInTheDocument()
      expect(screen.getByText("unit")).toBeInTheDocument()
      expect(screen.getByText("e2e")).toBeInTheDocument()
      expect(screen.getByText("happy_path")).toBeInTheDocument()
      expect(screen.getAllByText("performance#api-latency").length).toBeGreaterThanOrEqual(1)
    })

    it("shows uncovered badge for uncovered behaviors", () => {
      const spec = mockSpec({
        behaviors: [
          { name: "refresh", description: "Refresh", covered: false, test_types: [], category: "error_case", nfr_refs: [] },
        ],
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByText("uncovered")).toBeInTheDocument()
      expect(screen.getByText("error_case")).toBeInTheDocument()
    })
  })

  /// panel-info-dialog: Info button opens a dialog with spec description and motivation
  describe("panel-info-dialog", () => {
    it("shows info button next to spec name", () => {
      const spec = mockSpec({ name: "auth-command" })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )
      expect(screen.getByRole("button", { name: /spec info/i })).toBeInTheDocument()
    })

    it("opens dialog with description and motivation when available", async () => {
      const user = userEvent.setup()
      const spec = mockSpec({
        name: "auth-command",
        title: "Authentication Command",
        description: "Handles user authentication flows",
        motivation: "Users need secure login",
      })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )

      await user.click(screen.getByRole("button", { name: /spec info/i }))

      expect(screen.getByText("Authentication Command")).toBeInTheDocument()
      expect(screen.getByText("Handles user authentication flows")).toBeInTheDocument()
      expect(screen.getByText("Users need secure login")).toBeInTheDocument()
    })

    it("shows placeholder when description and motivation are not available", async () => {
      const user = userEvent.setup()
      const spec = mockSpec({ name: "auth-command" })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )

      await user.click(screen.getByRole("button", { name: /spec info/i }))

      expect(screen.getByText(/not yet available from the API/)).toBeInTheDocument()
    })

    it("uses spec name as dialog title when title field is not available", async () => {
      const user = userEvent.setup()
      const spec = mockSpec({ name: "auth-command" })
      render(
        <SpecSlidePanel spec={spec} isOpen={true} onClose={vi.fn()} />
      )

      await user.click(screen.getByRole("button", { name: /spec info/i }))

      // The dialog title should fall back to the spec name
      const dialog = screen.getByRole("dialog")
      expect(within(dialog).getByText("auth-command")).toBeInTheDocument()
    })
  })
})
