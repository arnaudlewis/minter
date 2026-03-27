import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, it, expect, vi } from "vitest"
import { NfrCardGrid } from "../NfrCardGrid"
import { mockNfr } from "@/test/fixtures"

describe("NfrCardGrid", () => {
  /// nfr-card-displays-summary: Each NFR card shows category, version, and constraint count
  describe("nfr-card-displays-summary", () => {
    it("shows NFR category name on the card", () => {
      const nfr = mockNfr({ category: "performance", version: "1.1.0" })
      render(<NfrCardGrid nfrs={[nfr]} onSelectNfr={vi.fn()} />)
      expect(screen.getByText("performance")).toBeInTheDocument()
    })

    it("shows NFR version on the card", () => {
      const nfr = mockNfr({ category: "performance", version: "1.1.0" })
      render(<NfrCardGrid nfrs={[nfr]} onSelectNfr={vi.fn()} />)
      expect(screen.getByText("v1.1.0")).toBeInTheDocument()
    })

    it("shows constraint count on the card", () => {
      const nfr = mockNfr({ constraint_count: 7 })
      render(<NfrCardGrid nfrs={[nfr]} onSelectNfr={vi.fn()} />)
      expect(screen.getByText(/7 constraints/)).toBeInTheDocument()
    })

    it("shows validation status icon for valid NFR", () => {
      const nfr = mockNfr({ validation_status: "Valid" })
      render(<NfrCardGrid nfrs={[nfr]} onSelectNfr={vi.fn()} />)
      const card = screen.getByTestId("nfr-card")
      const svgs = card.querySelectorAll("svg")
      const greenIcon = Array.from(svgs).find(
        (svg) => svg.classList.contains("text-emerald-400")
      )
      expect(greenIcon).toBeTruthy()
    })

    it("renders multiple NFR cards", () => {
      const nfrs = [
        mockNfr({ category: "performance", version: "1.1.0", constraint_count: 7 }),
        mockNfr({ category: "reliability", version: "1.0.0", constraint_count: 4 }),
      ]
      render(<NfrCardGrid nfrs={nfrs} onSelectNfr={vi.fn()} />)
      expect(screen.getByText("performance")).toBeInTheDocument()
      expect(screen.getByText("reliability")).toBeInTheDocument()
      expect(screen.getByText(/7 constraints/)).toBeInTheDocument()
      expect(screen.getByText(/4 constraints/)).toBeInTheDocument()
    })

    it("shows section header 'NFR Constraints'", () => {
      const nfr = mockNfr()
      render(<NfrCardGrid nfrs={[nfr]} onSelectNfr={vi.fn()} />)
      expect(screen.getByText("NFR Constraints")).toBeInTheDocument()
    })
  })

  /// nfr-card-invalid: An NFR file with parse errors shows error indicator on the card
  describe("nfr-card-invalid", () => {
    it("shows red error indicator for invalid NFR", () => {
      const nfr = mockNfr({
        category: "performance",
        validation_status: { Invalid: ["parse error at line 3"] },
        constraint_count: 0,
        constraints: [],
      })
      render(<NfrCardGrid nfrs={[nfr]} onSelectNfr={vi.fn()} />)
      const card = screen.getByTestId("nfr-card")
      const svgs = card.querySelectorAll("svg")
      const redIcon = Array.from(svgs).find(
        (svg) => svg.classList.contains("text-red-400")
      )
      expect(redIcon).toBeTruthy()
    })

    it("shows 0 constraints for invalid NFR", () => {
      const nfr = mockNfr({
        validation_status: { Invalid: ["parse error"] },
        constraint_count: 0,
        constraints: [],
      })
      render(<NfrCardGrid nfrs={[nfr]} onSelectNfr={vi.fn()} />)
      expect(screen.getByText(/0 constraints/)).toBeInTheDocument()
    })
  })

  /// click on card calls onSelectNfr
  describe("card click", () => {
    it("calls onSelectNfr when card is clicked", async () => {
      const user = userEvent.setup()
      const onSelectNfr = vi.fn()
      const nfr = mockNfr({ category: "performance" })
      render(<NfrCardGrid nfrs={[nfr]} onSelectNfr={onSelectNfr} />)
      await user.click(screen.getByText("performance"))
      expect(onSelectNfr).toHaveBeenCalledWith(nfr)
    })
  })

  /// empty state: no NFR cards renders nothing
  describe("empty state", () => {
    it("does not render section when nfrs array is empty", () => {
      render(<NfrCardGrid nfrs={[]} onSelectNfr={vi.fn()} />)
      expect(screen.queryByText("NFR Constraints")).not.toBeInTheDocument()
    })
  })
})
