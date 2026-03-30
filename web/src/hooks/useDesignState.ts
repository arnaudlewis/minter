import { useState, useEffect, useCallback } from "react"
import type { DesignSystem, WsMessage } from "@/types"
import { mockDesignSystem } from "@/test/mock-design"

export function useDesignState() {
  const [designSystem, setDesignSystem] = useState<DesignSystem | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchDesign = useCallback(async () => {
    try {
      const res = await fetch("/api/design")
      if (res.ok) {
        const data = (await res.json()) as DesignSystem
        setDesignSystem(data)
        setError(null)
      } else {
        // No design yet or error — fall back to mock for development
        setDesignSystem(mockDesignSystem)
        setError(null)
      }
    } catch {
      // Backend not available — fall back to mock data for development
      setDesignSystem(mockDesignSystem)
      setError(null)
    } finally {
      setLoading(false)
    }
  }, [])

  // Listen for design-update WebSocket messages
  useEffect(() => {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:"
    const ws = new WebSocket(`${protocol}//${window.location.host}/ws`)

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data as string) as WsMessage
        if (msg.type === "design-update") {
          setDesignSystem(msg.data)
          setError(null)
        }
      } catch {
        // Ignore malformed messages
      }
    }

    ws.onerror = () => {
      // WS errors are non-fatal for the design view
    }

    return () => {
      ws.close()
    }
  }, [])

  useEffect(() => {
    fetchDesign()
  }, [fetchDesign])

  return { designSystem, loading, error }
}
