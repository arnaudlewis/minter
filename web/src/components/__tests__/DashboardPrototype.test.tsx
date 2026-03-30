import { render, screen, within } from "@testing-library/react"
import { describe, it, expect } from "vitest"
import { DashboardPrototype } from "../design/DashboardPrototype"
import { mockDesignSystem } from "@/test/mock-design"

describe("DashboardPrototype", () => {
  /// design-preview-renders-dashboard: Dashboard renders with sidebar, header, metrics, table, and status panel
  describe("design-preview-renders-dashboard", () => {
    it("renders the dashboard container", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      expect(screen.getByTestId("design-dashboard")).toBeInTheDocument()
    })

    it("renders the sidebar with project name", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const sidebar = screen.getByTestId("design-sidebar")
      expect(sidebar).toBeInTheDocument()
      expect(within(sidebar).getByText("Minter")).toBeInTheDocument()
    })

    it("renders sidebar navigation items from layout config", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const sidebar = screen.getByTestId("design-sidebar")
      for (const item of mockDesignSystem.layout.sidebar!.nav_items) {
        expect(within(sidebar).getByText(item)).toBeInTheDocument()
      }
    })

    it("renders domain names in sidebar", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const sidebar = screen.getByTestId("design-sidebar")
      for (const domain of mockDesignSystem.spec_metadata!.domains) {
        expect(within(sidebar).getByText(domain.name)).toBeInTheDocument()
      }
    })

    it("renders the header with title", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      expect(screen.getByTestId("design-header")).toBeInTheDocument()
    })

    it("renders search input when has_search is true", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      expect(screen.getByText("Search specs...")).toBeInTheDocument()
    })

    it("renders metric cards section", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      expect(screen.getByTestId("design-metric-cards")).toBeInTheDocument()
    })

    it("displays metric card labels", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const cards = screen.getByTestId("design-metric-cards")
      // Labels are uppercase in metric cards
      expect(within(cards).getByText("Specifications")).toBeInTheDocument()
      expect(within(cards).getByText("Behaviors")).toBeInTheDocument()
      expect(within(cards).getByText("Entities")).toBeInTheDocument()
      expect(within(cards).getByText("Coverage")).toBeInTheDocument()
    })

    it("displays correct spec count in metric card", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const cards = screen.getByTestId("design-metric-cards")
      expect(
        within(cards).getByText(String(mockDesignSystem.spec_metadata!.total_spec_count))
      ).toBeInTheDocument()
    })

    it("displays correct behavior count in metric card", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const cards = screen.getByTestId("design-metric-cards")
      expect(
        within(cards).getByText(String(mockDesignSystem.spec_metadata!.total_behavior_count))
      ).toBeInTheDocument()
    })

    it("displays correct entity count in metric card", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const cards = screen.getByTestId("design-metric-cards")
      expect(
        within(cards).getByText(String(mockDesignSystem.spec_metadata!.total_entity_count))
      ).toBeInTheDocument()
    })

    it("renders the data table with spec metrics", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const table = screen.getByTestId("design-data-table")
      expect(table).toBeInTheDocument()
      expect(within(table).getByText("Name")).toBeInTheDocument()
      expect(within(table).getByText("Version")).toBeInTheDocument()
      expect(within(table).getByText("Status")).toBeInTheDocument()
    })

    it("renders spec names in the data table", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const table = screen.getByTestId("design-data-table")
      for (const spec of mockDesignSystem.spec_metadata!.spec_metrics) {
        expect(within(table).getByText(spec.name)).toBeInTheDocument()
      }
    })

    it("renders spec versions prefixed with v", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const table = screen.getByTestId("design-data-table")
      expect(within(table).getByText("v2.1.0")).toBeInTheDocument()
      expect(within(table).getByText("v1.3.0")).toBeInTheDocument()
    })

    it("renders the status panel", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      expect(screen.getByTestId("design-status-panel")).toBeInTheDocument()
    })

    it("renders status categories in status panel", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const panel = screen.getByTestId("design-status-panel")
      expect(within(panel).getByText("Valid")).toBeInTheDocument()
      expect(within(panel).getByText("Warnings")).toBeInTheDocument()
      expect(within(panel).getByText("Errors")).toBeInTheDocument()
    })
  })

  /// design-tokens-drive-styling: All visual properties come from design tokens, not hardcoded values
  describe("design-tokens-drive-styling", () => {
    it("applies sidebar background from palette shades", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const sidebar = screen.getByTestId("design-sidebar")
      // jsdom converts hex to rgb
      expect(sidebar.style.backgroundColor).toBe("rgb(30, 58, 138)")
    })

    it("applies sidebar width from layout config", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const sidebar = screen.getByTestId("design-sidebar")
      expect(sidebar.style.width).toBe(mockDesignSystem.layout.sidebar!.width)
    })

    it("applies font family from type scale", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const dashboard = screen.getByTestId("design-dashboard")
      // jsdom normalizes quotes in font-family
      expect(dashboard.style.fontFamily).toContain("Inter")
      expect(dashboard.style.fontFamily).toContain("system-ui")
      expect(dashboard.style.fontFamily).toContain("sans-serif")
    })

    it("renders in light theme (white background for content)", () => {
      render(<DashboardPrototype design={mockDesignSystem} />)
      const dashboard = screen.getByTestId("design-dashboard")
      expect(dashboard.style.backgroundColor).toBe("rgb(248, 250, 252)")
    })
  })

  /// design-sidebar-hidden: No sidebar renders when layout has no sidebar config
  describe("design-sidebar-hidden", () => {
    it("does not render sidebar when layout.sidebar is undefined", () => {
      const noSidebar = {
        ...mockDesignSystem,
        layout: {
          ...mockDesignSystem.layout,
          sidebar: undefined,
        },
      }
      render(<DashboardPrototype design={noSidebar} />)
      expect(screen.queryByTestId("design-sidebar")).not.toBeInTheDocument()
    })
  })
})
