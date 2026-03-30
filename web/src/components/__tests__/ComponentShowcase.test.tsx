import { render, screen, within } from "@testing-library/react"
import { describe, it, expect } from "vitest"
import { ComponentShowcase } from "../design/ComponentShowcase"
import { mockDesignSystem } from "@/test/mock-design"

describe("ComponentShowcase", () => {
  /// render-color-palette: Renders color swatches for all palette roles and shade ramps
  describe("render-color-palette", () => {
    it("renders the color palette section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-color-palette")).toBeInTheDocument()
    })

    it("displays primary color hex value", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-color-palette")
      expect(within(section).getByText(/Primary #2563eb/i)).toBeInTheDocument()
    })

    it("displays shade ramp values", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-color-palette")
      expect(within(section).getByText("#eff6ff")).toBeInTheDocument()
      expect(within(section).getByText("#1e3a8a")).toBeInTheDocument()
    })

    it("displays semantic color labels", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("semantic-success")).toBeInTheDocument()
      expect(screen.getByTestId("semantic-warning")).toBeInTheDocument()
      expect(screen.getByTestId("semantic-error")).toBeInTheDocument()
      expect(screen.getByTestId("semantic-info")).toBeInTheDocument()
    })
  })

  /// render-typography-showcase: Renders each type scale level at its actual size
  describe("render-typography-showcase", () => {
    it("renders the typography section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-typography")).toBeInTheDocument()
    })

    it("displays the font family", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-typography")
      expect(within(section).getByText(/'Inter', system-ui, sans-serif/)).toBeInTheDocument()
    })

    it("displays type scale sample texts", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-typography")
      expect(within(section).getByText("Design System")).toBeInTheDocument()
      expect(within(section).getByText("Caption text")).toBeInTheDocument()
    })
  })

  /// render-spacing-visualization: Renders visual spacing steps from the spacing grid
  describe("render-spacing-visualization", () => {
    it("renders the spacing section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-spacing")).toBeInTheDocument()
    })

    it("displays base unit", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-spacing")
      expect(within(section).getByText(/Base unit: 4px/)).toBeInTheDocument()
    })

    it("displays step values in px", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-spacing")
      expect(within(section).getByText("4px")).toBeInTheDocument()
      expect(within(section).getByText("64px")).toBeInTheDocument()
    })
  })

  /// render-navigation-component: Renders a navigation component styled with tokens
  describe("render-navigation-component", () => {
    it("renders the navigation section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-navigation")).toBeInTheDocument()
    })

    it("renders the nav preview container", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-nav-preview")).toBeInTheDocument()
    })

    it("renders project name in navigation", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const nav = screen.getByTestId("showcase-nav-preview")
      expect(within(nav).getByText("Minter")).toBeInTheDocument()
    })

    it("renders navigation items derived from spec names", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const nav = screen.getByTestId("showcase-nav-preview")
      // First domain spec names are transformed to title case
      expect(within(nav).getByText("Validate Command")).toBeInTheDocument()
      expect(within(nav).getByText("Watch Command")).toBeInTheDocument()
    })
  })

  /// render-metric-cards: Renders metric cards with real project numbers
  describe("render-metric-cards", () => {
    it("renders the metric cards section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-metric-cards")).toBeInTheDocument()
    })

    it("displays spec count", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-metric-cards")
      expect(within(section).getByText("29")).toBeInTheDocument()
    })

    it("displays behavior count", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-metric-cards")
      expect(within(section).getByText("653")).toBeInTheDocument()
    })

    it("displays entity count", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-metric-cards")
      expect(within(section).getByText("14")).toBeInTheDocument()
    })

    it("displays card labels", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-metric-cards")
      expect(within(section).getByText("Specs")).toBeInTheDocument()
      expect(within(section).getByText("Behaviors")).toBeInTheDocument()
      expect(within(section).getByText("Entities")).toBeInTheDocument()
    })
  })

  /// render-data-table: Renders a data table with real spec entries
  describe("render-data-table", () => {
    it("renders the data table section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-data-table")).toBeInTheDocument()
    })

    it("renders table column headers", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-data-table")
      expect(within(section).getByText("Name")).toBeInTheDocument()
      expect(within(section).getByText("Version")).toBeInTheDocument()
      expect(within(section).getByText("Behaviors")).toBeInTheDocument()
      expect(within(section).getByText("Status")).toBeInTheDocument()
    })

    it("renders spec names in rows", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-data-table")
      expect(within(section).getByText("validate-command")).toBeInTheDocument()
      expect(within(section).getByText("cli")).toBeInTheDocument()
    })

    it("renders version numbers prefixed with v", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-data-table")
      expect(within(section).getByText("v2.1.0")).toBeInTheDocument()
    })

    it("renders status badges", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-data-table")
      const validBadges = within(section).getAllByText("Valid")
      expect(validBadges.length).toBeGreaterThan(0)
    })
  })

  /// render-button-variants: Renders primary, secondary, ghost, and destructive buttons
  describe("render-button-variants", () => {
    it("renders the buttons section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-buttons")).toBeInTheDocument()
    })

    it("renders primary button", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-buttons")
      const btn = within(section).getByText("Primary")
      expect(btn).toBeInTheDocument()
      expect(btn.style.backgroundColor).toBeTruthy()
    })

    it("renders secondary button with border", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-buttons")
      const btn = within(section).getByText("Secondary")
      expect(btn).toBeInTheDocument()
    })

    it("renders ghost button", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-buttons")
      expect(within(section).getByText("Ghost")).toBeInTheDocument()
    })

    it("renders destructive button", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-buttons")
      expect(within(section).getByText("Destructive")).toBeInTheDocument()
    })
  })

  /// render-form-elements: Renders form inputs, selects, checkboxes, and radios
  describe("render-form-elements", () => {
    it("renders the form elements section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-form-elements")).toBeInTheDocument()
    })

    it("renders text input with label", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-form-elements")
      expect(within(section).getByText("Email Address")).toBeInTheDocument()
      expect(within(section).getByPlaceholderText("name@example.com")).toBeInTheDocument()
    })

    it("renders select dropdown", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-form-elements")
      expect(within(section).getByText("Category")).toBeInTheDocument()
      expect(within(section).getByText("Select a category...")).toBeInTheDocument()
    })

    it("renders checkboxes", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-form-elements")
      expect(within(section).getByText("Enable notifications")).toBeInTheDocument()
      expect(within(section).getByText("Auto-validate on save")).toBeInTheDocument()
    })

    it("renders radio buttons", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-form-elements")
      expect(within(section).getByText("Light")).toBeInTheDocument()
      expect(within(section).getByText("Dark")).toBeInTheDocument()
      expect(within(section).getByText("System")).toBeInTheDocument()
    })
  })

  /// render-alerts-badges: Renders alerts and badges using semantic colors
  describe("render-alerts-badges", () => {
    it("renders the alerts and badges section", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("showcase-alerts-badges")).toBeInTheDocument()
    })

    it("renders success alert", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("alert-success")).toBeInTheDocument()
    })

    it("renders warning alert", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("alert-warning")).toBeInTheDocument()
    })

    it("renders error alert", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("alert-error")).toBeInTheDocument()
    })

    it("renders info alert", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      expect(screen.getByTestId("alert-info")).toBeInTheDocument()
    })

    it("renders status badges", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const section = screen.getByTestId("showcase-alerts-badges")
      // Badge labels within the section
      expect(within(section).getByText("Primary")).toBeInTheDocument()
      expect(within(section).getByText("Accent")).toBeInTheDocument()
    })
  })

  /// showcase-styled-with-tokens: All visual properties come from design tokens
  describe("showcase-styled-with-tokens", () => {
    it("applies font family from type scale", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const showcase = screen.getByTestId("component-showcase")
      expect(showcase.style.fontFamily).toContain("Inter")
    })

    it("uses white background", () => {
      render(<ComponentShowcase design={mockDesignSystem} />)
      const showcase = screen.getByTestId("component-showcase")
      expect(showcase.style.backgroundColor).toBe("rgb(255, 255, 255)")
    })
  })

  /// empty-spec-metadata-fallback: Uses placeholder content when spec metadata is empty
  describe("empty-spec-metadata-fallback", () => {
    it("renders with fallback navigation items when no spec metadata", () => {
      const noMetadata = { ...mockDesignSystem, spec_metadata: null }
      render(<ComponentShowcase design={noMetadata} />)
      const nav = screen.getByTestId("showcase-nav-preview")
      expect(within(nav).getByText("Dashboard")).toBeInTheDocument()
      expect(within(nav).getByText("Settings")).toBeInTheDocument()
      expect(within(nav).getByText("Profile")).toBeInTheDocument()
    })

    it("renders metric cards with zero values when no spec metadata", () => {
      const noMetadata = { ...mockDesignSystem, spec_metadata: null }
      render(<ComponentShowcase design={noMetadata} />)
      const section = screen.getByTestId("showcase-metric-cards")
      const zeros = within(section).getAllByText("0")
      expect(zeros.length).toBe(3)
    })

    it("renders project name as fallback in navigation", () => {
      const noMetadata = { ...mockDesignSystem, spec_metadata: null }
      render(<ComponentShowcase design={noMetadata} />)
      const nav = screen.getByTestId("showcase-nav-preview")
      expect(within(nav).getByText("Project")).toBeInTheDocument()
    })
  })
})
