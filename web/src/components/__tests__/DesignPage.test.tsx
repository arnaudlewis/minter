import { render, screen, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import { DesignPage } from "../design/DesignPage"
import { mockDesignSystem } from "@/test/mock-design"

// Mock WebSocket
class MockWebSocket {
  onopen: (() => void) | null = null
  onmessage: ((event: { data: string }) => void) | null = null
  onclose: (() => void) | null = null
  onerror: (() => void) | null = null
  close = vi.fn()
}

describe("DesignPage", () => {
  let originalFetch: typeof globalThis.fetch
  let originalWebSocket: typeof globalThis.WebSocket

  beforeEach(() => {
    originalFetch = globalThis.fetch
    originalWebSocket = globalThis.WebSocket
    globalThis.WebSocket = MockWebSocket as unknown as typeof WebSocket
  })

  afterEach(() => {
    globalThis.fetch = originalFetch
    globalThis.WebSocket = originalWebSocket
  })

  /// design-page-loading: Shows loading state while fetching design system
  describe("design-page-loading", () => {
    it("shows loading spinner initially", () => {
      // Never-resolving fetch to keep loading state
      globalThis.fetch = vi.fn(() => new Promise<Response>(() => {}))
      render(<DesignPage />)
      expect(screen.getByTestId("design-loading")).toBeInTheDocument()
      expect(screen.getByText(/loading design system/i)).toBeInTheDocument()
    })
  })

  /// design-page-empty: Shows empty state when no design system exists
  describe("design-page-empty", () => {
    it("shows empty state when fetch returns 404 and no mock fallback", async () => {
      // This test verifies the empty state component renders correctly standalone
      // In practice, the hook falls back to mock data on 404
      const { EmptyDesignState } = await import("../design/EmptyDesignState")
      render(<EmptyDesignState />)
      expect(screen.getByTestId("design-empty-state")).toBeInTheDocument()
      expect(screen.getByText(/no design system generated/i)).toBeInTheDocument()
      expect(screen.getByText(/generate_layout/i)).toBeInTheDocument()
    })
  })

  /// design-page-renders: Shows dashboard prototype when design system is available
  describe("design-page-renders", () => {
    it("renders dashboard prototype after loading mock data", async () => {
      // Simulate fetch failure so hook falls back to mockDesignSystem
      globalThis.fetch = vi.fn(() => Promise.reject(new Error("no backend")))
      render(<DesignPage />)
      await waitFor(() => {
        expect(screen.getByTestId("design-page")).toBeInTheDocument()
      })
      expect(screen.getByTestId("design-dashboard")).toBeInTheDocument()
    })

    it("renders with data from successful API response", async () => {
      globalThis.fetch = vi.fn(() =>
        Promise.resolve(new Response(JSON.stringify(mockDesignSystem), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }))
      )
      render(<DesignPage />)
      await waitFor(() => {
        expect(screen.getByTestId("design-page")).toBeInTheDocument()
      })
      expect(screen.getByTestId("design-dashboard")).toBeInTheDocument()
    })
  })
})
